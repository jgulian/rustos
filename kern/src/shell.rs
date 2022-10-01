use stack_vec::StackVec;
use crate::console;

use crate::console::{kprint, kprintln, CONSOLE};

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs,
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>,
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// returns if the `exit` command is called.
pub fn shell(prefix: &str) -> ! {
    'shell_loop: loop {
        let mut command: [u8; 512] = [0; 512];
        let mut container= [""; 64];
        let mut i = 0;

        kprint!("{}", prefix);

        'command_loop: loop {
            let byte = CONSOLE.lock().read_byte();
            if byte == b'\n' || byte == b'\r' {
                kprintln!("");
                break 'command_loop;
            } else if byte == 8 || byte == 127 {
                if i > 0 {
                    CONSOLE.lock().write_byte(8);
                    CONSOLE.lock().write_byte(b' ');
                    CONSOLE.lock().write_byte(8);
                    i -= 1;
                }
            } else {
                CONSOLE.lock().write_byte(byte);
                command[i] = byte;
                i += 1;
                if i == 512 {
                    kprintln!("");
                    kprintln!("command too long");
                    continue 'shell_loop;
                }
            }
        }

        let command_str = core::str::from_utf8(&command[0..i]).expect("");
        match Command::parse(command_str, &mut container) {
            Ok(command) => {
                run_command(&command);
            }
            Err(err) => {
                match err {
                    Error::Empty => {}
                    Error::TooManyArgs => {kprintln!("too many args");}
                }
            }
        }
    }
}

fn run_command(command: &Command) {
    match command.path() {
        "echo" => {
            for i in 1..command.args.len() {
                kprint!("{} ", command.args[i]);
            }
            kprintln!("");
        }
        _ => {
            kprintln!("unknown command: {}", command.path());
        }
    }
}
