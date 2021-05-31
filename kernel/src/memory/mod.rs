//! Memory management subsystem.
//!
//! This module provides cross-platform facilities for managing
//! various memory-related chores: virtual memory, and having a heap
//!
//! # Virtual memory
//! In a modern computer, virtual memory is provided as a way to
//! protect a process' memory from modifications made by another one
//! (and some other reasons). This is done by creating a virtual address
//! space, divided in "pages", and mapping each page to a region of
//! physical memory, called "frame".
//!
//! Virtual memory handling is eased by this module through usage of
//! the [`PhysAddr`] and [`VirtAddr`] structs, which represent addresses,
//! and the [`FrameAllocator`] and [`Pager`] trait, which provide some
//! convenience functions.
//!
//! # Heap
//! For many applications, the size of some data can't be known in advance,
//! or even changes through the execution of the program. For cases like
//! this, stack allocation isn't enough, and so we must use a heap.
//!
//! A heap, though, needs to be initialized, and also needs an allocator
//! to give out sections of it that are not used by someone else. This
//! module sets everything up in the [`init_heap`] function

#[path = "../arch/x86_64/memory.rs"]
mod arch;

use types::{VirtAddr, PhysAddr, Pager};

use core::alloc::Layout;
use linked_list_allocator::LockedHeap;
pub use arch::{allocate_frame, init, FrameAllocImpl, PagerImpl, PAGE_SIZE};

/// The physical address of the heap
pub const HEAP_START: u64 = 0x_4444_4444_0000;
/// The size in bytes of the heap
pub const HEAP_SIZE: u64 = 100 * 1024; // 100 KiB

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}

/// Initialize a heap for the kernel, and set up the allocator
pub fn init_heap(mapper: &mut impl Pager) -> Option<()> {
    let page_range = {
        let heap_start = HEAP_START;
        let heap_end = heap_start + HEAP_SIZE - 1;

        (heap_start..heap_end).step_by(PAGE_SIZE)
    };

    for page in page_range {
        assert_eq!(page % PAGE_SIZE as u64, 0);

        let frame = allocate_frame()?;
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