.PHONY: all deps clean clean-all all-kernel

RUSTSRC = rust
export RUSTSRC

default: all

all: kernel # user # Disable user mode for now

deps:
	${MAKE} -C deps all

kernel: deps
	${MAKE} -C kernel all

user: deps
	${MAKE} -C user all

clean:
	${MAKE} -C kernel clean
	${MAKE} -C user clean

clean-all: clean
	${MAKE} -C deps clean

run%: kernel
	${MAKE} -C kernel $@
