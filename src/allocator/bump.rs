use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize, //指向下一个可用的地址
    allocations: usize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_usize: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_usize;
        self.next = heap_start;
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    /*
    获取对包装的分配器类型的可变引用。
    实例在方法结束之前保持锁定状态，
    因此在多线程上下文中不会发生数据争用
    */

    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock();

        let alloc_start = align_up(bump.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > bump.heap_end {
            ptr::null_mut()
        } else {
            bump.next = alloc_end;
            bump.allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock();

        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}

// unsafe impl GlobalAlloc for BumpAllocator {
//     unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
//         let alloc_start = self.next;
//         self.next += layout.size();
//         self.allocations += 1;
//         alloc_start as *mut u8
//     }

//     unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
//         todo!();
//     }
// }
