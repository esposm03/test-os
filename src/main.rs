#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rpi_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::boxed::Box;
use bootloader::{entry_point, BootInfo};
use rpi_os::println;
use x86_64::instructions::hlt;

entry_point!(main);
fn main(info: &'static BootInfo) -> ! {
    println!("Hello world!");
    rpi_os::init(info);

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
