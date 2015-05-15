### OS 1 ###
============

Implementation of a simple OS in Rust

I had originally intended not to have any library code in my kernel at all, but this is not possible with Rust yet. As a result I am linking in `libcore`, which contains the heart of Rust functionality. Special thanks to Krzysztof for helping me get that working!

See [krzysz00/rust-kernel](https://github.com/krzysz00/rust-kernel) for build instructions.

Thanks to
- Krzysztof Drewniak for build system and help
- AG for bootloader code
- OSDev wiki for general helpfulness
- rustboot (github.com/charliesome/rustboot)
