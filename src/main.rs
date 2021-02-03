#![no_std]
#![no_main]

use core::fmt::Write;

mod vga_buffer;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    write!(vga_buffer::WRITER.lock(), "Ciao Arianna\nn1 + 1 = {}", 1+1).unwrap();

    loop {}
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
