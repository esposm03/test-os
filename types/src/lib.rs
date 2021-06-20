#![no_std]

extern crate alloc;

use alloc::boxed::Box;

use spin::Mutex;

pub struct KernelState<P: Pager, F: FrameAllocator, V> {
    pub pager: Mutex<P>,
    pub frame_alloc: Mutex<F>,
    pub vga_buffer: V,
}

impl<P: Pager, F: FrameAllocator, V> KernelState<P, F, V> {
    pub fn allocate_frame(&self) -> PhysAddr {
        self.frame_alloc.lock().next().expect("All frames have been used")
    }
}

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
/// behaviour if it is not implemented correctly
pub unsafe trait Pager {
    /// Translate from virtual to physical addresses
    ///
    /// If the provided virtual address is not mapped to any
    /// frame, then `None` is returned
    fn translate(&self, addr: VirtAddr) -> Option<PhysAddr>;

    /// Create a mapping in the page table
    ///
    /// # Safety
    /// The caller must ensure that the frame given in the `to` argument
    /// is not already used, and also that nothing is stored in the page
    /// denoted by `addr`, unless everything is copied to the new location
    /// after the remapping
    unsafe fn map(&mut self, addr: VirtAddr, to: PhysAddr) -> Option<()>;
}

unsafe impl<P: Pager> Pager for Box<P> {
    fn translate(&self, addr: VirtAddr) -> Option<PhysAddr> {
        Pager::translate(self.as_ref(), addr)
    }

    unsafe fn map(&mut self, addr: VirtAddr, to: PhysAddr) -> Option<()> {
        Pager::map(self.as_mut(), addr, to)
    }
}

impl From<x86_64::PhysAddr> for PhysAddr {
    fn from(a: x86_64::PhysAddr) -> Self {
        Self(a.as_u64())
    }
}
impl From<PhysAddr> for x86_64::PhysAddr {
    fn from(a: PhysAddr) -> Self {
        x86_64::PhysAddr::new(a.0)
    }
}
impl From<x86_64::VirtAddr> for VirtAddr {
    fn from(a: x86_64::VirtAddr) -> Self {
        Self(a.as_u64())
    }
}
impl From<VirtAddr> for x86_64::VirtAddr {
    fn from(a: VirtAddr) -> Self {
        x86_64::VirtAddr::new(a.0)
    }
}
