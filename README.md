### OS 1 ###
============

Implementation of a simple OS in Rust

I had originally intended not to have any library code in my kernel at all, but this is not possible with Rust yet. As a result I am linking in `libcore` and `liballoc` which contain the heart of Rust functionality. Special thanks to Krzysztof for helping me get that set up!

Website [here](https://mark-i-m.github.com/os1)

###Features so far
* Heap allocation
* Preemptive multitasking
* VGA buffer management
* Reaper
* Concurrency Control
* Virtual Memory

###TODO
* User mode/system calls
* File system
* Process resources
    - Finish native semaphore implementation

###Requirements

* ```qemu```
* ```gcc```
* ```rust 1.6 (2015-11-08 nightly): curl -sSf https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly --date=2015-11-08```

###Build instructions

* ```git clone https://github.com/mark-i-m/os1.git```
* ```cd os1/```
* ```git submodule init && git submodule update```
* ```make rungraphic```

If you want better performance, rather than debuggability, compile with optimization level 3, rather than 0. This can be achieved by changing `common.mak`.

### To generate Rustdocs:

Run this in the `kernel` directory
```
rustdoc -o $DOC_OUTPUT --extern rlibc=../deps/librlibc.rlib --extern core=../deps/libcore.rlib --extern alloc=../deps/liballoc.rlib --target ../i686-unknown-elf.json -w html --no-defaults --passes strip-hidden --passes collapse-docs --passes unindent-comments lib.rs
```

Thanks to
- Krzysztof Drewniak for build system and help
- AG for bootloader code
- OSDev wiki for general helpfulness
- [rustboot](http://github.com/charliesome/rustboot)
