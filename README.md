### OS 1 ###
============

Implementation of a simple OS in Rust

I had originally intended not to have any library code in my kernel at all, but this is not possible with Rust yet. As a result I am linking in `libcore` and `liballoc` which contain the heart of Rust functionality. Special thanks to Krzysztof for helping me get that set up!

###Features so far
* Heap allocation
* Preemptive multitasking
* VGA buffer management
* Reaper
* Concurrency Control

###TODO
* Virtual Memory
* User mode/system calls
* File system

###Requirements

* ```qemu```
* ```gcc```
* ```rust 1.6 (2015-11-08 nightly): curl -sSf https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly --date=2015-11-08```

###Build instructions

* ```git clone https://github.com/mark-i-m/os1.git```
* ```cd os1/```
* ```git submodule init && git submodule update```
* ```make rungraphic```

Thanks to
- Krzysztof Drewniak for build system and help
- AG for bootloader code
- OSDev wiki for general helpfulness
- [rustboot](http://github.com/charliesome/rustboot)
