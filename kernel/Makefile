.PHONY: all clean

# rust libraries
DEPDIR = ../lib

# run configuration
QEMUEXTRA =
KERNELDEBUG =
KERNELSERIAL =

# by default build, but do not run
default: all

all: kernel.img

-include ../common.mak

# kernel build
kernel: ${OFILES} ${AFILES}
	${LD} --gc-sections -N -m elf_i386 -e start --section-start mbr=0x7c00 -Ttext=0x9000 -o kernel ${OFILES} --start-group ${AFILES} --end-group

# kernel run
# DO NOT ENABLE KVM!!! For some reason it causes weird crashes...
run: all
	qemu-system-x86_64 ${KERNELDEBUG} ${KERNELSERIAL} --serial mon:stdio -drive format=raw,file=kernel.img,index=2,media=disk ${QEMUEXTRA}

runtext: KERNELSERIAL = -nographic
runtext: run

rungraphic: RUSTOPT = -C opt-level=3
rungraphic: ASOPT = -O3
rungraphic: run

rundebug: KERNELDEBUG = -s -S
rundebug: clean runtext

# kernel clean
clean::
	rm -f kernel
