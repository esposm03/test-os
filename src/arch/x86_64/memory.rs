use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{self, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Translate},
};

use crate::memory::{self, FrameAllocator, Pager};

impl From<x86_64::PhysAddr> for memory::PhysAddr {
    fn from(a: x86_64::PhysAddr) -> Self {
        Self(a.as_u64())
    }
}

impl From<memory::PhysAddr> for x86_64::PhysAddr {
    fn from(a: memory::PhysAddr) -> Self {
        x86_64::PhysAddr::new(a.0)
    }
}

impl From<x86_64::VirtAddr> for memory::VirtAddr {
    fn from(a: x86_64::VirtAddr) -> Self {
        Self(a.as_u64())
    }
}

impl From<memory::VirtAddr> for x86_64::VirtAddr {
    fn from(a: memory::VirtAddr) -> Self {
        x86_64::VirtAddr::new(a.0)
    }
}

pub unsafe fn init(phys_offset: memory::VirtAddr) -> impl Pager {
    let level_4_table = active_l4_page_table(x86_64::VirtAddr::new(phys_offset.0));
    PagerImpl(OffsetPageTable::new(level_4_table, phys_offset.into()), Fr)
}

/// Given the physical offset of "map all memory" tables, defined by
/// the bootloader, return a reference to the active l4 page table.
unsafe fn active_l4_page_table(phys_offset: x86_64::VirtAddr) -> &'static mut PageTable {
    let (l4_table_frame, _) = Cr3::read();

    let phys = l4_table_frame.start_address();
    let virt = phys_offset.as_u64() + phys.as_u64();
    let page_table_ptr = virt as *mut PageTable;

    &mut *page_table_ptr
}

pub struct PagerImpl(OffsetPageTable<'static>, FrameAllocImpl);

impl Pager for PagerImpl {
    fn translate(&self, addr: memory::VirtAddr) -> Option<memory::PhysAddr> {
        self.0.translate_addr(addr.into()).map(|a| a.into())
    }

    unsafe fn map(&mut self, addr: memory::VirtAddr, to: memory::PhysAddr) -> Option<()> {
        let page = Page::<paging::Size4KiB>::containing_address(addr.into());
        let frame = PhysFrame::containing_address(to.into());
        let flags = paging::PageTableFlags::PRESENT | paging::PageTableFlags::WRITABLE;
        let frame_allocator = &mut self.1;

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
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        FrameAllocImpl {
            memory_map,
            next: 0,
        }
    }

    pub fn usable_frames(&self) -> impl Iterator<Item = memory::PhysAddr> {
        self.memory_map
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .map(|r| r.range.start_addr()..r.range.end_addr())
            .flat_map(|r| r.step_by(4096))
            .map(|addr| addr - (addr % 4096))
            .map(|i| memory::PhysAddr(i))
    }
}

impl memory::FrameAllocator for FrameAllocImpl {
    fn next(&mut self) -> Option<memory::PhysAddr> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

unsafe impl paging::FrameAllocator<paging::Size4KiB> for FrameAllocImpl {
    fn allocate_frame(&mut self) -> Option<PhysFrame<paging::Size4KiB>> {
        Some(PhysFrame::containing_address(self.next()?.into()))
    }
}
