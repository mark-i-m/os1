#OS 1

Implementation of a simple OS in Rust

For more information about the implementation, Rustdocs, a screenshot, and a
bootable .img file, visit the website [here](https://mark-i-m.github.com/os1).

For those that want to build from source, instructions are below.

##Building

###Requirements

* `qemu`
* `gcc`
* Rust 1.12 (2016-07-25 nightly)

  It needs to be nightly Rust, because the project uses unstable language features.

  `curl -sSf https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly --date=2016-07-25

###To build:

* `git clone https://github.com/mark-i-m/os1.git`
* `cd os1/`
* `git submodule init && git submodule update`
* `make rungraphic`

### To generate Rustdocs:

Run this in the `kernel` directory with `$DOC_OUTPUT` set to the directory where
you want the output:

```bash
rustdoc -o $DOC_OUTPUT -w html \
    --extern rlibc=../lib/librlibc.rlib \
    --extern core=../lib/libcore.rlib \
    --extern alloc=../lib/liballoc.rlib \
    --target ../i686-unknown-elf.json \
    --no-defaults --passes strip-hidden \
    --passes collapse-docs --passes unindent-comments lib.rs
```

##Thanks to

- Krzysztof Drewniak for build system
- AG for original bootloader code
- OSDev wiki for general helpfulness
