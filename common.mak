# Choose tools
RUSTC = rustc
CC = gcc

AR = ar
LD = ld

OBJCOPY = objcopy
DD = dd

# DO NOT USE -O0. If kernel.img is > 64K, the BIOS will not load it
RUSTFLAGS += --target=../i686-unknown-elf.json -L. -L${DEPDIR} -g -C opt-level=1 -Z no-landing-pads
ASFLAGS += -m32 -g -O1

vpath %.rs %.s %.S $(sort $(dir $(wildcard ./*) $(wildcard ./**/*)))
RUSTFILES = $(notdir $(wildcard **/*.rs))
SFILES = $(notdir $(wildcard *.S) $(wildcard *.s))

OFILES = $(subst .s,.o,$(subst .S,.o,$(SFILES)))
AFILES = libasmcode.a librustcode.a

BOOTFILES = $(sort $(filter boot%,${OFILES}))
NON_BOOTFILES = $(filter-out boot%,${OFILES})

# Make rules
.PHONY: clean

%.o: %.S
	${CC} ${ASFLAGS} -c -o $@ $<

%.o: %.s
	${CC} ${ASFLAGS} -c -o $@ $<

libasmcode.a: ${OFILES}
	${AR} cr $@ ${NON_BOOTFILES}

librustcode.a: ${RUSTFILES}
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
