### OS 1 ###
============

Implementation of a simple OS in Rust

Thanks to
- rustboot (github.com/charliesome/rustboot)
- AG for bootloader code and makefile
- OSDev wiki for general helpfulness

Building and running
====================
- To build: `make`
- To run with qemu: `make run`
- To run with qemu and graphics on: `make rungraphics`
- To attach with gdb:
    - `qemu-system-x86_64 -s -S -nographic --serial mon:stdio -hdc kernel/kernel.img`
    - Then in another window:
        ```
        $ gdb kernel/kernel
        (gdb) target remote localhost:1234
        (gdb) # define breakpoint, etc
        (gdb) cont
        ```
