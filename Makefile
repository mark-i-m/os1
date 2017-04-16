.PHONY: all kernel mkfs user clean clean-all

default: kernel user mkfs

# build
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
run: kernel user mkfs
	qemu-system-i386 ${KERNELDEBUG} ${KERNELSERIAL} ${QEMUEXTRA} --serial mon:stdio -hdc kernel/kernel.img -hdd mkfs/hdd.img

runtext: KERNELSERIAL = -nographic
runtext: run

rungraphic:
	make run RUSTOPT="--release" ASOPT="-O3"

rundebug: KERNELDEBUG = -s -S
rundebug: clean run

# clean
clean:
	${MAKE} -C kernel clean
	${MAKE} -C mkfs clean
	${MAKE} -C user clean

clean-all: clean
	${MAKE} -C kernel clean-all
