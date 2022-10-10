use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use shim::{io, ioerr};
use shim::path::{Path, PathBuf};

use stack_vec::StackVec;
use fat32::traits::FileSystem;
use crate::{console, FILESYSTEM};

use pi::atags::Atags;

//use fat32::traits::FileSystem;
//use fat32::traits::{Dir, Entry};

use crate::console::{kprint, kprintln, CONSOLE};
//use crate::ALLOCATOR;
//use crate::FILESYSTEM;

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
/// never returns.
pub fn shell(prefix: &str) -> ! {
    use fat32::traits::{FileSystem, Entry, Dir};
    let mut cwd = PathBuf::from("/");


    'shell_loop: loop {
        let mut command: [u8; 512] = [0; 512];
        let mut container = [""; 64];
        let mut i = 0;


        kprint!("({}) {}", cwd.to_str().unwrap_or("err"), prefix);

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
                match run_command(&command, cwd.clone()) {
                    Some(new_cwd) => cwd = new_cwd,
                    _ => {}
                }
            }
            Err(err) => {
                match err {
                    Error::Empty => {}
                    Error::TooManyArgs => { kprintln!("too many args"); }
                }
            }
        }
    }
}

fn run_command(command: &Command, cwd: PathBuf) -> Option<PathBuf> {
    match command.path() {
        "echo" => {
            for i in 1..command.args.len() {
                kprint!("{} ", command.args[i]);
            }
            kprintln!("");
        }
        "pwd" => {
            kprintln!("{}", cwd.to_str().unwrap_or("unknown working dir"));
        }
        "cd" => {
            if command.args.len() < 2 {
                kprintln!("not enough args for cd");
                return None;
            }

            use fat32::traits::{Entry, Dir};

            let mut path = cwd.clone();
            path.push(PathBuf::from(command.args[1]));
            let path_name = path_name(path.clone());

            let dir = match FILESYSTEM.open(path.clone()) {
                Err(e) => {
                    kprintln!("unable to open {}", path_name);
                    return None;
                }
                Ok(dir) => {
                    match dir.into_dir() {
                        None => {
                            kprintln!("{} is not a directory", path_name);
                            return None;
                        }
                        Some(real_dir) => {
                            return Some(path);
                        },
                    }
                }
            };
        }
        "ls" => {
            use fat32::traits::{Entry, Dir};

            let mut path = cwd.clone();
            let mut show_hidden = false;
            for arg in command.args[1..].iter() {
                match *arg {
                    "-a" => {
                        show_hidden = true
                    }
                    _ => {
                        path.push(PathBuf::from(arg));
                        break;
                    }
                }
            }

            let path_name = path_name(path.clone());

            let dir = match FILESYSTEM.open(path) {
                Err(e) => {
                    kprintln!("unable to open {}", path_name);
                    return None;
                }
                Ok(dir) => {
                    match dir.into_dir() {
                        None => {
                            kprintln!("{} is not a directory", path_name);
                            return None;
                        }
                        Some(real_dir) => real_dir,
                    }
                }
            };

            let entries = match dir.entries() {
                Err(e) => {
                    kprintln!("unable to read entries of {}", path_name);
                    return None;
                }
                Ok(e) => {
                    e
                }
            };

            for entry in entries {
                if !entry.name().starts_with(".") || show_hidden {
                    kprintln!("{}", entry.name());
                }
            }
        }
        "cat" => {
            use io::Read;
            use fat32::traits::{Entry, File};

            if command.args.len() < 2 {
                kprintln!("not enough args for cat");
                return None;
            }

            let path = cwd.join(PathBuf::from(command.args[1]));
            let path_name = path_name(path.clone());

            let mut file = match FILESYSTEM.open(path) {
                Err(e) => {
                    kprintln!("unable to open {}", path_name);
                    return None;
                }
                Ok(file) => {
                    match file.into_file() {
                        None => {
                            kprintln!("{} is not a file", path_name);
                            return None;
                        }
                        Some(file) => file,
                    }
                }
            };

            let mut data = vec![0u8; file.file_size as usize];
            let n = match file.read(data.as_mut_slice()) {
                Err(e) => {
                    kprintln!("unable to read file");
                    return None;
                }
                Ok(n) => n,
            };
            let str: String = data.iter().map(|x| (*x as char)).collect();

            kprintln!("{}", str);
        }
        _ => {
            kprintln!("unknown command: {}", command.path());
        }
    }

    None
}

fn path_name(path: PathBuf) -> String {
    String::from(path.to_str().unwrap_or("err"))
}