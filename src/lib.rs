#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::BootInfo;
use memory::BootInfoFrameAllocator;
use x86_64::{VirtAddr, instructions::hlt, structures::paging::{FrameAllocator, Mapper, Size4KiB}};

pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod vga_buffer;
pub mod allocator;

pub fn init(info: &'static BootInfo) -> (impl Mapper<Size4KiB>, impl FrameAllocator<Size4KiB>) {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    let frame_alloc = unsafe { BootInfoFrameAllocator::init(&info.memory_map) };
    let mapper = unsafe { memory::init(VirtAddr::new(info.physical_memory_offset)) };

    (mapper, frame_alloc)
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T: Fn()> Testable for T {
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(0x10);
}

#[cfg(test)]
bootloader::entry_point!(test_start);

/// Entry point for `cargo test`
#[cfg(test)]
fn test_start(_: &bootloader::BootInfo) -> ! {
    init();
    test_main();
    loop {
        hlt()
    }
}

#[cfg_attr(test, panic_handler)]
pub fn test_panic_handler(info: &core::panic::PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(0x11);
}

pub fn exit_qemu(exit_code: u32) -> ! {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code);
    }

    loop {
        hlt()
    }
}
