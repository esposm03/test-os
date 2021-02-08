#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(rpi_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use rpi_os::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    rpi_os::init();

    #[cfg(test)]
    test_main();

    // unsafe {
    //     *(0xdeadbeef as *mut u64) = 42;
    // }

    fn stack_overflow() {
        stack_overflow();
    }

    stack_overflow();

    loop {}
}

#[panic_handler]
fn panic(i: &core::panic::PanicInfo) -> ! {
    println!("{}", i);

    loop {}
}
