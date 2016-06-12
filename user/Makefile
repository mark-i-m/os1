DEBUGFLAGS = -ggdb -O0
CFLAGS = -std=c99 -m32 -ffreestanding -nostdlib -nodefaultlibs -Wall -Werror $(DEBUGFLAGS)

FILELIST=$(wildcard *.c)
EXECLIST=$(patsubst %.c, files/%, $(FILELIST))

DEPS=crt0.o

all: $(EXECLIST)

%.o: %.S
	gcc -c -m32 $^ 

%.o: %.c
	gcc -c -MD $(CFLAGS) $^

files/%: %.o $(DEPS)
	ld -N -m elf_i386 -e start -Ttext=0x80000000 -o $@ $^

clean:
	rm -f *.o
	rm -f *.d
	rm -f $(EXECLIST)

-include *.d
