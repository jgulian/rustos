#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(optin_builtin_traits)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

pub mod console;
pub mod mutex;
pub mod shell;

use console::kprintln;

// FIXME: You need to add dependencies here to
// test your drivers (Phase 2). Add them as needed.

use pi::uart::MiniUart;

fn kmain() -> ! {
    // FIXME: Start the shell.
    let mut mini_uart = MiniUart::new();

    loop {
        let c = mini_uart.read_byte();
        mini_uart.write_byte(c);
    }
}
