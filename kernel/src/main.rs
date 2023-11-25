#![no_std]
#![no_main]

use core::arch::asm;

#[no_mangle]
pub extern "sysv64" fn kernel_main() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
