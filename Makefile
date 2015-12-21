.PHONY: all lib clean clean-all

RUSTSRC = rust
export RUSTSRC

default: rungraphic

lib:
	${MAKE} -C lib all

clean:
	${MAKE} -C kernel clean
	${MAKE} -C user clean

clean-all: clean
	${MAKE} -C lib clean

run%: lib
	${MAKE} -C kernel $@
