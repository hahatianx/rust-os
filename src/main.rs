#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod vga_buffer;
mod serial;

use core::panic::PanicInfo;
use blog_os::halt_loop;

#[no_mangle]
pub extern "C" fn _start() -> ! {

    println!("Hello World{}", "!");

    blog_os::init();

    #[cfg(test)]
    test_main();

    halt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    halt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(_info);
}
