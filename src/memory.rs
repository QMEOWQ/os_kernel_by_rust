use x86_64::structures::paging::{Mapper, Page, PageTable, RecursivePageTable};
use x86_64::{PhysAddr, VirtAddr};

// 返回一个对活动的4级表的可变引用。
// 这个函数是不安全的，因为调用者必须保证完整的物理内存在传递的
// `physical_memory_offset`处被映射到虚拟内存。另外，这个函数
// 必须只被调用一次，以避免别名"&mut "引用（这是未定义的行为）。
pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe ptr
}

pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    translate_addr_inner(addr, physical_memory_offset)
}

// 这个函数是安全的，可以限制`unsafe`的范围，
fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    use x86_64::registers::control::Cr3;
    use x86_64::structures::paging::page_table::FrameError;

    let (level_4_table_frame, _) = Cr3::read();

    let table_indices = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];
    let mut frame = level_4_table_frame;

    // 遍历多级页表
    for &index in &table_indices {
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        // 读取条目并更新frame
        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported!"),
        };
    }

    Some(frame.start_address() + u64::from(addr.page_offset()))
}

// // 创建一个可递归访问的四级页表
// let level_4_table = [...];
// let level_4_table_ptr = level_4_table_addr as *mut PageTable;
// let recursive_page_table = unsafe {
//     let level_4_table = &mut *level_4_table_ptr;
//     RecursivePageTable::new(level_4_table).unwrap();
// }

// // 检索给定虚拟地址的物理地址
// let addr: u64 = [...];
// let addr = VirtAddr::new(addr);
// let page: Page = Page::containing_address(addr);

// // 进行翻译
// let frame = recursive_page_table.translate_page(page);
// frame.map(|frame| frame.start_address() + u64::from(addr.page_offset()))

// 创建一个可递归访问的四级页表
// 你想访问其对应的页表的虚拟地址
// let addr: usize = […];

// let r = 0o777; // 递归索引
// let sign = 0o177777 << 48; // 符号扩展

// // 检索我们要翻译的地址的页表索引
// let l4_idx = (addr >> 39) & 0o777; // level 4 索引
// let l3_idx = (addr >> 30) & 0o777; // level 3 索引
// let l2_idx = (addr >> 21) & 0o777; // level 2 索引
// let l1_idx = (addr >> 12) & 0o777; // level 1 索引
// let page_offset = addr & 0o7777;

// // 计算页表的地址
// let level_4_table_addr =
//     sign | (r << 39) | (r << 30) | (r << 21) | (r << 12);
// let level_3_table_addr =
//     sign | (r << 39) | (r << 30) | (r << 21) | (l4_idx << 12);
// let level_2_table_addr =
//     sign | (r << 39) | (r << 30) | (l4_idx << 21) | (l3_idx << 12);
// let level_1_table_addr =
//     sign | (r << 39) | (l4_idx << 30) | (l3_idx << 21) | (l2_idx << 12);
