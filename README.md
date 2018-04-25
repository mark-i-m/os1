# OS 1

Implementation of a simple OS in Rust

For more information about the implementation, Rustdocs, a screenshot, and a
bootable .img file, visit the website [here](https://mark-i-m.github.com/os1).

For those that want to build from source, instructions are below.

## Building

### Requirements

* `qemu`
* `gcc`
* Nightly Rust + Cargo. I am using Rust 1.27.0-nightly (ac3c2288f 2018-04-18)

  It needs to be _nightly_ Rust, because the project uses unstable language features.

  `curl https://sh.rustup.rs -sSf | sh`

* `xargo` via `cargo install xargo` (used to cross-compile)

### To build:

* `git clone https://github.com/mark-i-m/os1.git`
* `cd os1/`
* `make rungraphic`

### To generate Rustdocs:

Run this in the `kernel` directory. Then open `target/doc/kernel/index.html` in
a web browser.

```bash
$ cargo rustdoc -- --no-defaults --passes "collapse-docs" --passes "unindent-comments"
```

## Thanks to

- AG for original bootloader code
- OSDev wiki for general helpfulness
