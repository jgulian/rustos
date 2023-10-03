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

Currently, the project is on `1.68.0-nightly`. I'm working on making it easier
to run on general (more) hardware, but for now qemu is recommended with the
settings in the Makefile.

## Roadmap

### Spring 2023
* pseudo filesystem
* Use RAII to handle freeing, allocating, and cow
* Scheduler refactor
  * Add more than one scheduler
* Promised
  * Zero Page Initilization
  * Ticketing Scheduler
  * Scheduler adjustment through file system
  * Clone system call
  * thread_create, thread_wait, locking
  * UID get, set
  * chown, chmod
  * create user / login user
  * Require login on boot

### Backburner

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

\* Asterisks suggest features are in a partially complete state, but require more work or coverage testing.