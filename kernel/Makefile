.PHONY: all clean

.SECONDARY: kernel.img kernel.bin

DEPDIR = ../deps

default: all

all: kernel.img

-include ../common.mak

# kernel build
kernel: ${BOOTFILES} ${AFILES}
	${LD} --gc-sections -N -m elf_i386 -e start --section-start mbr=0x7c00 -Ttext=0x9000 -o kernel mbr.o --start-group ${AFILES} --end-group

# DO NOT ENABLE KVM!!! For some reason it causes weird crashes...
# kernel run
runtext: all
	qemu-system-x86_64 -nographic --serial mon:stdio -drive format=raw,file=kernel.img,index=2,media=disk

rungraphic: all
	qemu-system-x86_64 --serial mon:stdio -drive format=raw,file=kernel.img,index=2,media=disk

rundebug: clean all
	qemu-system-x86_64 -s -S -nographic --serial mon:stdio -drive format=raw,file=kernel.img,index=2,media=disk

# kernel clean
clean::
	rm -f kernel
