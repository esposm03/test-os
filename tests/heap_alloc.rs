#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rpi_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rpi_os::memory::HEAP_SIZE;

use alloc::{boxed::Box, vec::Vec};

entry_point!(main);

fn main(info: &'static BootInfo) -> ! {
    rpi_os::init(info);
    test_main();

    loop {
        x86_64::instructions::hlt()
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rpi_os::test_panic_handler(info)
}

#[test_case]
fn simple_alloc() {
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);
}

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn many_boxes() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}
