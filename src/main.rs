#![no_std]
#![no_main]

use core::fmt::Write;

mod vga_buffer;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Ciao mondo");
    println!("1+1 = {}", 1+1);

    panic!("I'm Gayyy");

    loop {}
}

#[panic_handler]
fn panic(i: &core::panic::PanicInfo) -> ! {
    println!("{}", i);

    loop {}
}
