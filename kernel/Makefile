# Choose tools
RUSTC = rustc
CC = gcc

AR = ar
LD = ld

OBJCOPY = objcopy
DD = dd

# rust libraries
LIBS = ../lib

# Choose options
RUSTOPT = -C opt-level=0 -g
ASOPT = -O0 -g
RUSTCACHE = ../build_cache/debug

RUSTFLAGS += --target=../i686-unknown-elf.json -L. -L${LIBS} ${RUSTOPT} -Z no-landing-pads -Z incremental=${RUSTCACHE}
ASFLAGS += -m32 ${ASOPT}

# Lists of files
RUSTFILES = $(shell find -name '*.rs')
SFILES = $(wildcard *.S) $(wildcard *.s)

OFILES = $(subst .s,.o,$(subst .S,.o,$(SFILES)))
AFILES = libos1.a

# General rules
.PHONY: all clean

# by default build, but do not run
all: kernel.img

%.o: %.S
	${CC} ${ASFLAGS} -c -o $@ $<

%.o: %.s
	${CC} ${ASFLAGS} -c -o $@ $<

libos1.a: ${RUSTFILES}
	${RUSTC} ${RUSTFLAGS} lib.rs

%.bin: %
	${OBJCOPY} -O binary $< $@

%.img: %.bin
	${DD} if=$< of=$@ bs=512 conv=sync

# kernel build
kernel: ${OFILES} ${AFILES}
	${LD} --gc-sections -N -m elf_i386 -e start --section-start mbr=0x7c00 -Ttext=0x9000 -o kernel ${OFILES} --start-group ${AFILES} --end-group

clean:
	rm -f *.o
	rm -f *.a
	rm -f *.img
	rm -f *.bin
	rm -f kernel
