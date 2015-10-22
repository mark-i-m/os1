### OS 1 ###
============

Implementation of a simple OS in Rust

I had originally intended not to have any library code in my kernel at all, but this is not possible with Rust yet. As a result I am linking in `libcore`, which contains the heart of Rust functionality. Special thanks to Krzysztof for helping me get that working!

###Features so far
* Heap allocation
* Preemptive multitasking
* VGA buffer management

###TODO
* Reaper
* Concurrency Control
* Virtual Memory
* User mode/system calls
* File system

###Requirements

* ```qemu```
* ```gcc```
* ```rust 1.1``` (2015-05-10 nightly version or after), by this command line ```curl -sSf https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly --date=2015-05-10```

###Build instructions

* ```git clone https://github.com/mark-i-m/os1.git```
* ```cd os1/```
* ```git submodule init && git submodule update```
* ```make rungraphic```

Thanks to
- Krzysztof Drewniak for build system and help
- AG for bootloader code
- OSDev wiki for general helpfulness
- rustboot (github.com/charliesome/rustboot)

