# RustOS: A Kernel for the Raspberry Pi

This project was done out of an interest in operating systems. As a base I used a previously offered course on
operating systems at Georgia Tech, which is no longer offered. I completed all parts of the course excluding networking.
I decided not to do networking because I would like to implement network drivers in safe rust instead of writing safe
wrappers for unsafe c.

## Roadmap

* Make the file system writable
    * Move from the current c library to a rust library
* Add page swapping mechanisms
* Allow the stack to grow downwards (ie outside one page of memory)
* Introduce a feature-rich IPC structure.
* Support ELF file format
* Add user space concurrency features (including locks, CVs, and semaphores)
* Work on a hosted hypervisor
* Networking
* Use cargo fuzz and miri to libraries for security.
