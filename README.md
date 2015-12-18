#OS 1

Implementation of a simple OS in Rust

There is a comprehensive website [here](https://mark-i-m.github.com/os1) that
includes a .img file for those who want to just boot the OS, an explanation of
the implementation, and Rustdocs of the code.

For those that want to build from source, instructions are below.

##Building

###Requirements

* `qemu`
* `gcc`
* Rust 1.6 (2015-11-08 nightly)

..It needs to be nightly Rust, because the project uses unstable language features.
..`curl -sSf https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly --date=2015-11-08`

###Build instructions

* `git clone https://github.com/mark-i-m/os1.git`
* `cd os1/`
* `git submodule init && git submodule update`
* `make rungraphic`

### To generate Rustdocs:

Run this in the `kernel` directory with `$DOC_OUTPUT` set to the directory where
you want the output:

```bash
rustdoc -o $DOC_OUTPUT -w html \
    --extern rlibc=../deps/librlibc.rlib \
    --extern core=../deps/libcore.rlib \
    --extern alloc=../deps/liballoc.rlib \
    --target ../i686-unknown-elf.json \
    --no-defaults --passes strip-hidden \
    --passes collapse-docs --passes unindent-comments lib.rs
```

Thanks to
- Krzysztof Drewniak for build system
- AG for original bootloader code
- OSDev wiki for general helpfulness
