.PHONY: all clean

RUSTC = rustc

RSFLAGS = --target=../i686-unknown-elf.json -L . --crate-type=rlib -C opt-level=3 -Z no-landing-pads

all: librlibc.rlib liballoc.rlib liballoc_system.rlib

libcore.rlib:
	${RUSTC} ${RSFLAGS} rust/src/libcore/lib.rs

liballoc.rlib: libcore.rlib
	${RUSTC} ${RSFLAGS} --cfg feature=\"external_funcs\" rust/src/liballoc/lib.rs

liballoc_system.rlib: alloc_system.rs libcore.rlib
	${RUSTC} ${RSFLAGS} --cfg feature=\"external_funcs\" $<

librlibc.rlib: rlibc.rs libcore.rlib
	${RUSTC} ${RSFLAGS} $<

clean::
	rm -f *.rlib

