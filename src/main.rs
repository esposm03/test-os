#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(rpi_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use rpi_os::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello world!");

    #[cfg(test)]
    test_main();

    loop {}
}

#[test_case]
fn trivial_assertions() {
    assert_eq!(1+1, 2);
}
