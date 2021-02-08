#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(rpi_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use rpi_os::println;
use x86_64::instructions::hlt;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello world!");

    rpi_os::init();

    #[cfg(test)]
    test_main();

    println!("Kernel did not crash");

    loop { hlt() }
}

#[panic_handler]
fn panic(i: &core::panic::PanicInfo) -> ! {
    println!("{}", i);

    loop { hlt() }
}
