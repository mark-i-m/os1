.PHONY: all clean

.SECONDARY: kernel.img kernel.bin

DEPDIR = ../deps

default: all

all: kernel.img

-include ../common.mak

# kernel build
kernel: ${BOOTFILES} ${AFILES}
	${LD} --gc-sections -N -m elf_i386 -e start --section-start mbr=0x7c00 -Ttext=0x9000 -o kernel mbr.o --start-group ${AFILES} --end-group

# kernel run
runtext: all
	qemu-system-x86_64 -enable-kvm -nographic --serial mon:stdio -hdc kernel.img

rungraphic: all
	qemu-system-x86_64 -enable-kvm --serial mon:stdio -hdc kernel.img

# kernel clean
clean::
	rm -f kernel
