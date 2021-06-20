#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use kernel::println;

use x86_64::instructions::hlt;
use bootloader::{entry_point, BootInfo};

entry_point!(main);
fn main(info: &'static BootInfo) -> ! {
    println!("Hello world!");
    kernel::init(info);

    #[cfg(test)]
    test_main();
    loop {
        hlt()
    }
}

#[panic_handler]
fn panic(i: &core::panic::PanicInfo) -> ! {
    println!("{}", i);

    loop {
        hlt()
    }
}

unsafe fn jmp(addr: *const u8) {
    let addr: fn() = core::mem::transmute(addr);
    addr();
}
