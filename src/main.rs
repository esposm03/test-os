#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rpi_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use rpi_os::{allocator, println};
use x86_64::instructions::hlt;
use alloc::boxed::Box;

entry_point!(main);
fn main(info: &'static BootInfo) -> ! {
    println!("Hello world!");
    let (mut mapper, mut frame_allocator) = rpi_os::init(info);

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap creation failed");

    let a = Box::new(10);

    println!("a: {}", a);

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
