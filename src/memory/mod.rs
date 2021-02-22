/// Memory management subsystem.
///
/// This module aims to provide memory-management facilities in a
/// cross-platform way. Currently, a `FrameAllocator`, a `Pager`,
/// and, in the future, an `Allocator` will be provided

#[path = "../arch/x86_64/memory.rs"]
mod arch;

pub use arch::{init, FrameAllocImpl, PagerImpl};

pub struct VirtAddr(pub u64);
pub struct PhysAddr(pub u64);

/// TODO: Handle deallocations
pub trait FrameAllocator {
    fn next(&mut self) -> Option<PhysAddr>;
}

/// Trait providing mapping, and address translation
pub trait Pager {
    fn translate(&self, addr: VirtAddr) -> Option<PhysAddr>;
    unsafe fn map(&mut self, addr: VirtAddr, to: PhysAddr) -> Option<()>;
}
