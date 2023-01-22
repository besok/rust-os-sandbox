#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os_sandbox::test_runner)]
#![reexport_test_harness_main = "test_main"]
use core::panic::PanicInfo;
use rust_os_sandbox::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os_sandbox::test_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    #[cfg(test)]
    test_main();

    loop {}
}

#[test_case]
fn test_println() {
    println!("test_println output");
    assert_eq!(1, 1);
}
