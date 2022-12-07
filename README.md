# RustOS: A Kernel for the Raspberry Pi

This project was done out of an interest in operating systems. As a base I used
a previously offered course on operating systems at Georgia Tech, which is no
longer offered. I completed all parts of the course excluding networking. I
decided not to do networking because I would like to implement network drivers
in safe rust instead of writing safe wrappers for unsafe c.

## Current State

The current focus of the project over the past week has been moving away from
the course and towards modern Rust features. After this, I will proceed with
working on

## Installing

Currently, the project is on `1.67.0-nightly`. I'm working on making it easier
to run on general (more) hardware, but for now qemu is recommended with the
settings in the Makefile.

## Features

* Kernel Allocator
* FAT32 Filesystem support (read and write)
* Preemptive Scheduling
* Virtual Memory
* Virtual File System
* Exception Calls (`exit`, `open`, `read`, `write`, `sbrk`, `fork`, `exec`, `wait`, and more)
* User programs (`cat`, `echo`, `shell` or `sh`, and more)

## Roadmap

* Make the file system writable*
    * Move from the current c library to a rust library
* Add page swapping mechanisms
* Allow the stack to grow downwards (ie outside one page of memory)
* Use smaller pages
* Introduce a feature-rich IPC structure.
* Support ELF file format
* Add user space concurrency features (including locks, CVs, and semaphores)
* Work on a hosted hypervisor
* Networking
* Use cargo fuzz and miri to libraries for security.
* Update bootloader
* Create a mkfs tool to make filesystem without needing to worry about OS/architecture (like xv6-riscv)
* Use cargo clippy

\* Asterisks suggest features are in a partially complete state, but require more work or coverage testing.