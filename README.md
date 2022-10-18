# RustOS: A Kernel for the Raspberry Pi

This project was done out of an interest in operating systems. As a base I used a previously offered course on
operating systems at Georgia Tech, which is no longer offered. Currently, I have completed the parts from the
course. I now plan to move on to expand the project in my own ways as outlined below.

## Roadmap

* Make the file system writable
    * Move from the current c library to a rust library
* Add page swapping mechanisms
* Allow the stack to grow downwards
* Introduce a feature-rich IPC structure.
* Support ELF file format
* Add user space concurrency features (including locks, CVs, and semaphores)
* Work on a hosted hypervisor
