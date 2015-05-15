.PHONY: all clean

DEPDIR = ../deps

default: all

-include ../common.mak

all: kernel.img

kernel: ${BOOTFILES} ${AFILES}
	${LD} --gc-sections -N -m elf_i386 -e start --section-start mbr=0x7c00 -Ttext=0x9000 -o kernel mbr.o --start-group ${AFILES} --end-group

clean::
	rm -f kernel

runtext: all
	qemu-system-x86_64 -enable-kvm -nographic --serial mon:stdio -hdc kernel.img
 
rungraphic: all
	qemu-system-x86_64 -enable-kvm --serial mon:stdio -hdc kernel.img
