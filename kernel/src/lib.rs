#![no_std]
#![feature(
    abi_x86_interrupt,
    alloc_error_handler,
    custom_test_frameworks,
)]

#![cfg_attr(test, no_main)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod vga_buffer;

use types::{KernelState, VirtAddr};

use bootloader::BootInfo;
use spin::{Mutex, Once};
use x86_64::instructions::hlt;

pub static KERNEL_STATE: Once<KernelState<memory::PagerImpl, memory::FrameAllocImpl, ()>> = Once::new();

#[track_caller]
pub fn kernel_state() -> &'static KernelState<memory::PagerImpl, memory::FrameAllocImpl, ()> {
    KERNEL_STATE.get().expect("KERNEL_STATE has not been initialized yet")
}

/// Initialize all of the kernel's subsystems (such as 
/// interrupt handling, memory management, serial, vga)
pub fn init(info: &'static BootInfo) {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    let (pager, frame_alloc) = unsafe {
        memory::init(
            VirtAddr(info.physical_memory_offset),
            &info.memory_map,
        )
    };

    let frame_alloc = Mutex::new(frame_alloc);
    let pager = Mutex::new(pager);

    KERNEL_STATE.call_once(|| KernelState {
        pager,
        vga_buffer: (),
        frame_alloc,
    });

    memory::init_heap().expect("Heap creation failed");
}

/// A test runner for the kernel
pub fn test_runner(tests: &[&dyn Fn()]) {
    pub trait Testable {
        fn run(&self);
    }

    impl<T: Fn()> Testable for T {
        fn run(&self) {
            serial_print!("{}...\t", core::any::type_name::<T>());
            self();
            serial_println!("[ok]");
        }
    }

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
