FILES=../user/files/root $(shell find ../user/files -type f -not -name "root") $(shell find ../kernel/ -type f -name '*.rs')

all: mkfs hdd.img

mkfs: mkfs.rs Makefile
	rustc -o mkfs mkfs.rs

hdd.img: mkfs Makefile $(FILES)
	@./mkfs $(FILES)

clean:
	rm -f mkfs hdd.img
