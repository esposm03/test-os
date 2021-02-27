//! Memory management subsystem.
//!
//! This module aims to provide memory-management facilities in a
//! cross-platform way. Currently, a `FrameAllocator`, a `Pager`,
//! and, in the future, an `Allocator` will be provided

#[path = "../arch/x86_64/memory.rs"]
mod arch;

pub use arch::{allocate_frame, init, FrameAllocImpl, PagerImpl};

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

/// The single source of truth about frames to be used
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

/// Virtual memory mapping, and address virtual-physical address translation
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
