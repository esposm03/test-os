use crate::{kernel_state, memory};
use types::{FrameAllocator, Pager};

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{self, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Translate},
};

/// The size of a page (and frame) on this architecture
pub const PAGE_SIZE: usize = 4096;

/// Init the memory subsystem
///
/// # Safety
///
/// The caller must ensure that the passed-in memory map is correct (as in, every
/// memory page marked as USABLE must actually be unused)
pub unsafe fn init(
    phys_offset: memory::VirtAddr,
    mem_map: &'static MemoryMap,
) -> (PagerImpl, FrameAllocImpl) {
    // TODO: Figure out a way to pass platform-specific info

    // Obtain the level 4 page table
    let (l4_table_frame, _) = Cr3::read();
    let phys = l4_table_frame.start_address();
    let virt = phys_offset.0 + phys.as_u64();
    let page_table_ptr = virt as *mut PageTable;
    let level_4_table = &mut *page_table_ptr;

    let frame_alloc = FrameAllocImpl::init(mem_map);
    let pager = PagerImpl(OffsetPageTable::new(level_4_table, phys_offset.into()));

    (pager, frame_alloc)
}

/// Reserve a frame for use
pub fn allocate_frame() -> Option<memory::PhysAddr> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut lock = kernel_state().frame_alloc.lock();
        lock.next()
    })
}

pub struct PagerImpl(OffsetPageTable<'static>);

unsafe impl Pager for PagerImpl {
    fn translate(&self, addr: memory::VirtAddr) -> Option<memory::PhysAddr> {
        self.0.translate_addr(addr.into()).map(|a| a.into())
    }

    unsafe fn map(&mut self, addr: memory::VirtAddr, to: memory::PhysAddr) -> Option<()> {
        crate::println!("Mapping 0x{:x} -> 0x{:x}", addr.0, to.0);

        let page = Page::<paging::Size4KiB>::containing_address(addr.into());
        let frame = PhysFrame::containing_address(to.into());
        let flags = paging::PageTableFlags::PRESENT | paging::PageTableFlags::WRITABLE;

        let lock = &mut kernel_state().frame_alloc.lock();
        let frame_allocator: &mut FrameAllocImpl = &mut *lock;

        self.0
            .map_to(page, frame, flags, frame_allocator)
            .ok()?
            .flush();

        Some(())
    }
}

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct FrameAllocImpl {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl FrameAllocImpl {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the passed memory map is, in fact, valid.
    /// Also, it must not be called more than once, as that would create two frame
    /// allocators giving out the same frames.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        FrameAllocImpl {
            memory_map,
            next: 0,
        }
    }

    /// Return an iterator over unallocated frames
    pub fn usable_frames(&self) -> impl Iterator<Item = memory::PhysAddr> {
        self.memory_map
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .map(|r| r.range.start_addr()..r.range.end_addr())
            .flat_map(|r| r.step_by(4096))
            .map(|addr| addr - (addr % 4096))
            .map(memory::PhysAddr)
    }
}

unsafe impl FrameAllocator for FrameAllocImpl {
    fn next(&mut self) -> Option<memory::PhysAddr> {
        let frame = self.usable_frames().nth(self.next);
        assert_eq!(frame.unwrap().0 % 4096, 0);
        self.next += 1;
        frame
    }
}

unsafe impl paging::FrameAllocator<paging::Size4KiB> for FrameAllocImpl {
    fn allocate_frame(&mut self) -> Option<PhysFrame<paging::Size4KiB>> {
        Some(PhysFrame::containing_address(self.next()?.into()))
    }
}
