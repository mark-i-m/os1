all: hdd.img

mkfs: mkfs.rs
	rustc -o mkfs mkfs.rs

hdd.img: mkfs
	./mkfs

clean:
	rm -f mkfs
