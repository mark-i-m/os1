# Choose tools
RUSTC = rustc
CC = gcc

AR = ar
LD = ld

OBJCOPY = objcopy
DD = dd

# Choose options
# If kernel.img is > 64K, the BIOS will not load it, so if it is crashing, use -O1, not -O0
RUSTOPT = -C opt-level=0 -g 
ASOPT = -O0 -g 

RUSTFLAGS += --target=../i686-unknown-elf.json -L. -L${DEPDIR} ${RUSTOPT} -Z no-landing-pads
ASFLAGS += -m32 ${ASOPT}

# Lists of files
RUSTFILES = $(shell find -name '*.rs')
SFILES = $(wildcard *.S) $(wildcard *.s)

OFILES = $(subst .s,.o,$(subst .S,.o,$(SFILES)))
AFILES = libos1.a

# General rules
.PHONY: clean

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

clean::
	rm -f *.o
	rm -f *.a
	rm -f *.img
	rm -f *.bin
