use core::alloc::Layout;

use linked_list_allocator::LockedHeap;

use crate::memory::{FrameAllocator, Pager, VirtAddr};

pub const HEAP_START: u64 = 0x_4444_4444_0000;
pub const HEAP_SIZE: u64 = 100 * 1024; // 100 KiB
pub const PAGE_SIZE: usize = 4096;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}

pub fn init_heap(mapper: &mut impl Pager, frame_allocator: &mut impl FrameAllocator) -> Option<()> {
    let page_range = {
        let heap_start = HEAP_START;
        let heap_end = heap_start + HEAP_SIZE - 1;

        (heap_start..heap_end).step_by(PAGE_SIZE)
    };

    for page in page_range {
        assert_eq!(page % PAGE_SIZE as u64, 0);

        let frame = frame_allocator.next()?;
        let page = VirtAddr(page);

        unsafe { mapper.map(page, frame)? }
    }

    unsafe {
        ALLOCATOR
            .lock()
            .init(HEAP_START as usize, HEAP_SIZE as usize);
    }

    Some(())
}
