#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::{max, min};

use kernel_api::{OsError, OsResult, print, println};
use kernel_api::file::File;
use kernel_api::syscall::{execute, exit, fork, open, time, wait};
use shim::io::Read;

use crate::user::get_arguments;

mod user;

fn parse(bytes: &[u8]) -> Option<(usize, usize, usize)> {
    let string = String::from_utf8_lossy(bytes);
    let mut lines = string.split('\n');

    Some((
        lines
            .next()?
            .split(' ')
            .skip(1)
            .next()?
            .parse::<usize>()
            .ok()?,
        lines
            .next()?
            .split(' ')
            .skip(1)
            .next()?
            .parse::<usize>()
            .ok()?,
        lines
            .next()?
            .split(' ')
            .skip(1)
            .next()?
            .parse::<usize>()
            .ok()?,
    ))
}

fn main() {
    match fork().unwrap() {
        None => {
            let sub_command_args: Vec<&'static str> = get_arguments().skip(1).collect();
            let sub_command = sub_command_args.join("\0");
            //TODO: this off by 1 is a bug, fix it
            execute(&sub_command.as_bytes(), &[]).unwrap();
            exit().unwrap();
        }
        Some(child) => {
            let start_time = time().unwrap();
            let mut data: Vec<(usize, usize, usize)> = Vec::new();
            let total_number = 2400; // TODO: find a ds to do this instead
            let mut allocator = File::new(open("/allocator").unwrap());
            let mut buffer = [0u8; 256];
            'sampling: loop {
                let waited = wait(child, Some(0)).unwrap();
                if let Some(_) = waited {
                    break 'sampling;
                }

                let read = allocator
                    .read(&mut buffer)
                    .expect("unable to read from allocator");
                if let Some(allocator_data) = parse(&buffer[..read]) {
                    if data.len() == total_number {
                        for i in (0..total_number).step_by(total_number / 80).rev() {
                            data.remove(i);
                        }
                    }
                    data.push(allocator_data);
                }
            }

            let stop_time = time().unwrap();
            let total_time = stop_time - start_time;
            println!("Total Millis: {}", total_time.as_millis());
            if let None = print_heap_data(data) {
                println!("failed to connect enough allocator data");
            }
        }
    }
}

fn print_heap_data(data: Vec<(usize, usize, usize)>) -> Option<()> {
    let chunk_size = data.len() / 80;
    if chunk_size == 0 {
        return None;
    }

    let (_, max_count, _) = data.iter().copied().max_by_key(|(_, count, _)| *count)?;
    let (_, min_count, _) = data.iter().copied().min_by_key(|(_, count, _)| *count)?;
    let (max_size, _, _) = data.iter().copied().max_by_key(|(size, _, _)| *size)?;
    let (min_size, _, _) = data.iter().copied().min_by_key(|(size, _, _)| *size)?;

    let scale = 10;

    let mut averages: Vec<(f32, f32)> = data
        .chunks(chunk_size)
        .map(|samples| {
            let total_size: usize = samples.iter().map(|(size, _, _)| *size).sum();
            let average_size = total_size / samples.len();
            let size_magnitude =
                (average_size - min_size) as f32 / (max_size - min_size) as f32 * scale as f32;
            //println!("size: {}", size_magnitude);

            let total_count: usize = samples.iter().map(|(_, count, _)| *count).sum();
            let average_count = total_count / samples.len();
            let count_magnitude =
                (average_count - min_count) as f32 / (max_count - min_count) as f32 * scale as f32;
            (size_magnitude, count_magnitude)
        })
        .collect();

    for _ in 0..80 {
        print!("_");
    }
    print!(" {}", max_size);

    for i in (0..scale).rev() {
        println!();
        for j in 0..80 {
            let difference = max(((averages[j].0 - i as f32) * 8 as f32) as i32, 0);
            let char = match difference {
                0 => ' ',
                1 => '▁',
                2 => '▂',
                3 => '▃',
                4 => '▄',
                5 => '▅',
                6 => '▆',
                7 => '▇',
                _ => '█',
            };

            print!("{}", char);
        }
    }
    println!(" {}", min_size);

    Some(())
}
