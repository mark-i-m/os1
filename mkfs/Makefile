all: mkfs hdd.img

mkfs: mkfs.rs Makefile
	rustc -o mkfs mkfs.rs

hdd.img: mkfs Makefile
	@./mkfs $(shell find ../kernel/ -type f -name '*.rs')

clean:
	rm -f mkfs hdd.img
