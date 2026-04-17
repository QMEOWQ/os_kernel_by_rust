use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll, Waker};
use spin::Mutex;

static CURRENT_TICK: AtomicU64 = AtomicU64::new(0);
static SLEEPERS: Mutex<Vec<SleeperEntry>> = Mutex::new(Vec::new());

#[derive(Clone)]
struct SleeperEntry {
    wake_tick: u64,
    waker: Waker,
}

pub fn current_tick() -> u64 {
    CURRENT_TICK.load(Ordering::Relaxed)
}

pub fn tick() {
    let tick_value = CURRENT_TICK.fetch_add(1, Ordering::Relaxed) + 1;
    let mut wakers_to_wake: Vec<Waker> = Vec::new();

    {
        let mut sleepers = SLEEPERS.lock();
        let mut index = 0;
        while index < sleepers.len() {
            if sleepers[index].wake_tick <= tick_value {
                let entry = sleepers.swap_remove(index);
                wakers_to_wake.push(entry.waker);
            } else {
                index += 1;
            }
        }
    }

    for waker in wakers_to_wake {
        waker.wake();
    }
}

pub fn sleep_ticks(ticks: u64) -> Sleep {
    let current = current_tick();
    let delta = if ticks == 0 { 1 } else { ticks };
    Sleep {
        wake_tick: current.saturating_add(delta),
    }
}

pub struct Sleep {
    wake_tick: u64,
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        if current_tick() >= self.wake_tick {
            return Poll::Ready(());
        }

        let mut sleepers = SLEEPERS.lock();
        if let Some(existing_entry) = sleepers
            .iter_mut()
            .find(|entry| entry.waker.will_wake(context.waker()))
        {
            existing_entry.wake_tick = self.wake_tick;
            existing_entry.waker = context.waker().clone();
        } else {
            sleepers.push(SleeperEntry {
                wake_tick: self.wake_tick,
                waker: context.waker().clone(),
            });
        }

        Poll::Pending
    }
}
