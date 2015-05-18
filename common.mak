.PHONY: clean

RUSTC = rustc
CC = gcc

AR = ar
LD = ld
OBJCOPY = objcopy
DD = dd

RUSTFLAGS += --target=../i686-unknown-elf.json -L. -L${DEPDIR} -g -C opt-level=0 -Z no-landing-pads
ASFLAGS += -m32

RUSTFILES = $(notdir $(wildcard *.rs) $(wildcard interrupts/*.rs))
SFILES = $(notdir $(wildcard *.S) $(wildcard *.s))

OFILES = $(subst .s,.o,$(subst .S,.o,$(SFILES)))
AFILES = libasmcode.a librustcode.a

BOOTFILES = $(sort $(filter boot%,${OFILES}))
NON_BOOTFILES = $(filter-out boot%,${OFILES})

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
