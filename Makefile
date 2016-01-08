.PHONY: all lib kernel mkfs user clean clean-all

default: lib kernel user mkfs

# build
lib:
	${MAKE} -C lib

kernel:
	${MAKE} -C kernel

mkfs:
	${MAKE} -C mkfs

user:
	${MAKE} -C user

# run

# run configuration
QEMUEXTRA =
KERNELDEBUG =
KERNELSERIAL =

# DO NOT ENABLE KVM!!! For some reason it causes weird crashes...
run: lib kernel user mkfs
	qemu-system-i386 ${KERNELDEBUG} ${KERNELSERIAL} --serial mon:stdio -hdc kernel/kernel.img -hdd mkfs/hdd.img

runtext: KERNELSERIAL = -nographic
runtext: run

rungraphic:
	make run RUSTOPT="-C opt-level=3" ASOPT="-O3"

rundebug: KERNELDEBUG = -s -S
rundebug: clean runtext

# clean
clean:
	${MAKE} -C kernel clean
	${MAKE} -C mkfs clean
	${MAKE} -C user clean

clean-all: clean
	${MAKE} -C lib clean
