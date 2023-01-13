#![no_main]

use hardware::bcm2835::uart::mini_uart::MiniUart;
use hardware::peripheral::character::CharacterDevice;

#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    let mut mini_uart = MiniUart::new();

    loop {
        mini_uart.write_byte(mini_uart.read_byte().expect("")).expect("");
    }
}
