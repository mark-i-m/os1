.PHONY: all clean

DEPDIR = .
RUSTC = rustc
RUSTFLAGS_LIBS = --target=../i686-unknown-elf.json -L . --crate-type=rlib -C opt-level=3 -Z no-landing-pads

default: all

all: librlibc.rlib liballoc.rlib liballoc_system.rlib 

libcore.rlib: ../i686-unknown-elf.json
	${RUSTC} ${RUSTFLAGS_LIBS} ${RUSTSRC}/src/libcore/lib.rs

liballoc.rlib: libcore.rlib
	${RUSTC} ${RUSTFLAGS_LIBS} --cfg feature=\"external_funcs\" ${RUSTSRC}/src/liballoc/lib.rs

liballoc_system.rlib: alloc_system.rs libcore.rlib
	${RUSTC} ${RUSTFLAGS_LIBS} --cfg feature=\"external_funcs\" $< 

librlibc.rlib: rlibc.rs libcore.rlib
	${RUSTC} ${RUSTFLAGS_LIBS} $<

clean::
	rm -f *.rlib

