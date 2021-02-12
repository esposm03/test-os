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
use x86_64::{instructions::hlt, VirtAddr};

pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod vga_buffer;

pub fn init(info: &'static BootInfo) {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&info.memory_map) };
    let mut mapper = unsafe { memory::init(VirtAddr::new(info.physical_memory_offset)) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap creation failed");
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
fn test_start(info: &'static bootloader::BootInfo) -> ! {
    init(info);
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
