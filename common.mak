RUSTC = rustc
CC = gcc

RSFLAGS = --target i686-unknown-linux-gnu -L . -L libcore 
RSFILES = $(wildcard *.rs)
SFILES = $(wildcard *.S) $(wildcard *.s)

OFILES = $(subst .rs,.o,$(RSFILES)) $(subst .s,.o,$(subst .S,.o,$(SFILES)))

# keep all files
.SECONDARY :

%.o :  Makefile %.rs
	$(RUSTC) $(RSFLAGS) --emit obj --emit dep-info --crate-type lib $*.rs

%.o :  Makefile %.S
	$(CC) -MD -m32 -c $*.S

%.o :  Makefile %.s
	$(CC) -MD -m32 -c $*.s

%.bin : Makefile %
	objcopy -O binary $* $*.bin

%.img : Makefile %.bin
	dd if=$*.bin of=$*.img bs=512 conv=sync

clean ::
	rm -f *.img
	rm -f *.bin
	rm -f *.o
	rm -f *.d

-include *.d
