use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use kernel_api::syscall::exit;
use shim::{io, ioerr};
use shim::path::{Path, PathBuf};

use stack_vec::StackVec;
use fat32::traits::{FileSystem};
use fat32::vfat::{Dir, Entry, File};
use crate::{console, FILESYSTEM};

use pi::atags::Atags;

//use fat32::traits::FileSystem;
//use fat32::traits::{Dir, Entry};

use crate::console::{kprint, kprintln, CONSOLE};
use crate::fs::PiVFatHandle;
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

pub struct Shell {
    prefix: String,
    cwd: PathBuf,
    open: bool,
}

impl Shell {
    pub fn new(prefix: &str) -> Self {
        Shell {
            prefix: prefix.to_string(),
            cwd: PathBuf::from("/"),
            open: true
        }
    }

    pub fn run(&mut self) {
        'shell_loop: while self.open {
            let mut command: [u8; 512] = [0; 512];
            let mut container = [""; 64];
            let mut i = 0;


            kprint!("({}) {} ", path_name(&self.cwd), self.prefix);

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
                    self.run_command(&command)
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

    fn run_command(&mut self, command: &Command) {
        match command.path() {
            "echo" => {
                self.echo(command);
            }
            "pwd" => {
                self.pwd();
            }
            "cd" => {
                self.cd(command);
            }
            "ls" => {
                self.ls(command);
            }
            "cat" => {
                self.cat(command);
            }
            "exit" => {
                self.exit();
            }
            _ => {
                kprintln!("unknown command: {}", command.path());
            }
        }
    }

    fn echo(&self, command: &Command) {
        for i in 1..command.args.len() {
            kprint!("{} ", command.args[i]);
        }
        kprintln!("");
    }

    fn pwd(&self) {
        kprintln!("{}", self.cwd.to_str().unwrap_or("unknown working dir"));
    }

    fn cd(&mut self, command: &Command) {
        if command.args.len() < 2 {
            kprintln!("not enough args for cd");
            return;
        }

        use fat32::traits::{Entry, Dir};

        let mut path = self.cwd.clone();
        path.push(PathBuf::from(command.args[1]));

        if open_dir(&path).is_some() {
            self.cwd = path;
        }
    }

    fn ls(&self, command: &Command) {
        use fat32::traits::{Entry, Dir};

        let mut path = self.cwd.clone();
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

        let dir = match open_dir(&path) {
            Some(dir) => dir,
            None => return,
        };

        let entries = match dir.entries() {
            Err(e) => {
                kprintln!("unable to read entries of {}", path_name(&path));
                return;
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

    fn cat(&self, command: &Command) {
        use io::Read;
        use fat32::traits::{Entry, File};

        if command.args.len() < 2 {
            kprintln!("not enough args for cat");
            return;
        }

        let path = self.cwd.join(PathBuf::from(command.args[1]));
        let mut file = match open_file(&path) {
            Some(file) => file,
            None => return,
        };

        let mut data = vec![0u8; file.file_size as usize];
        let n = match file.read(data.as_mut_slice()) {
            Err(e) => {
                kprintln!("unable to read file");
                return;
            }
            Ok(n) => n,
        };
        let str: String = data.iter().map(|x| (*x as char)).collect();

        kprintln!("{}", str);
    }

    fn exit(&mut self) {
        self.open = false;
    }
}

fn open(path: &PathBuf) -> Option<Entry<PiVFatHandle>> {
    match FILESYSTEM.open(path.clone()) {
        Err(e) => {
            kprintln!("unable to open {}", path_name(&path));
            None
        }
        Ok(dir) => {
            Some(dir)
        }
    }
}

fn open_dir(path: &PathBuf) -> Option<Dir<PiVFatHandle>> {
    use fat32::traits::Entry;
    match open(path)?.into_dir() {
        None => {
            kprintln!("{} is not a directory", path_name(&path));
            None
        }
        Some(real_dir) => {
            Some(real_dir)
        },
    }
}

fn open_file(path: &PathBuf) -> Option<File<PiVFatHandle>> {
    use fat32::traits::Entry;
    match open(path)?.into_file() {
        None => {
            kprintln!("{} is not a file", path_name(&path));
            None
        }
        Some(real_file) => {
            Some(real_file)
        },
    }
}

fn path_name(path: &PathBuf) -> String {
    path.clone().to_str().unwrap_or("err").to_string()
}