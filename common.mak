RSFLAGS = --target i686-unknown-linux-gnu --emit obj

RSFILES = $(wildcard *.rs)
SFILES = $(wildcard *.S) $(wildcard *.s)

OFILES = $(subst .rs,.o,$(RSFILES)) $(subst .s,.o,$(subst .S,.o,$(SFILES)))

# keep all files
.SECONDARY :

%.o :  Makefile %.rs
	rustc $(RSFLAGS) --crate-type lib -o $@ $*.rs

%.o :  Makefile %.S
	gcc -MD -m32 -c $*.S

%.o :  Makefile %.s
	gcc -MD -m32 -c $*.s

%.bin : Makefile %
	objcopy -O binary $* $*.bin

%.img : Makefile %.bin
	dd if=$*.bin of=$*.img bs=512 conv=sync

clean ::
	rm -f *.img
	rm -f *.bin
	rm -f *.o
	rm -f *.d
