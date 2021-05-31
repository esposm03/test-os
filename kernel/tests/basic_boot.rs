#![no_std]
#![no_main]

use rpi_os::{exit_qemu, println};

#[no_mangle]
extern "C" fn _start() -> ! {
    println!("Hello world");
    exit_qemu(0x10);
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rpi_os::test_panic_handler(info)
}
