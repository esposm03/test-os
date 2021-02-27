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

use core::alloc::Layout;

pub use arch::{allocate_frame, init, FrameAllocImpl, PagerImpl, PAGE_SIZE};
use linked_list_allocator::LockedHeap;

/// A virtual address - it doesn't correspond to a location
/// in memory, but must be translated to one.
///
/// Architecture-specific code should implement `From<VirtAddr> for T`
/// and `From<T> for VirtAddr`, and then convert between the two using
/// `.into()`
pub struct VirtAddr(pub u64);

/// A physical address, which can be directly retrieved from memory
///
/// Architecture-specific code should implement `From<VirtAddr> for T`
/// and `From<T> for VirtAddr`, and then convert between the two using
/// `.into()`
#[derive(Clone, Copy)]
pub struct PhysAddr(pub u64);

/// An allocator for frames, taking care of returning usable ones
///
/// When creating mappings from virtual memory addresses to physical ones,
/// we should be sure not to choose an invalid address, or one that is already
/// used. This is exactly the reason this trait exist.
///
/// # Safety
/// Implementing this trait is unsafe, as it is possible to cause undefined
/// behaviour by returning a frame that is already in use by some other code
pub unsafe trait FrameAllocator {
    // TODO: Handle deallocations
    /// Allocate a frame, and return its address
    fn next(&mut self) -> Option<PhysAddr>;
}

/// Virtual memory mapping, and virtual-physical address translation
///
/// This trait provides a way to create virtual memory pages pointing to
/// physical locations, and to translate virtual addresses to physical ones.
///
/// # Safety
/// This trait is unsafe to implement, as it is easy to cause undefined
/// behaviour unsafety if it is not implemented correctly
pub unsafe trait Pager {
    /// Translate from virtual to phisical addresses
    ///
    /// If the provided virtual address is not mapped to any
    /// frame, then `None` is returned
    fn translate(&self, addr: VirtAddr) -> Option<PhysAddr>;

    /// Create a mapping in the page table
    ///
    /// # Safety
    /// The called must ensure that the frame given in the `to` argument
    /// is not already used, and also that nothing is store in the page
    /// denoted by `addr`, unless everything is copied to the new location
    /// after the remapping
    unsafe fn map(&mut self, addr: VirtAddr, to: PhysAddr) -> Option<()>;
}

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