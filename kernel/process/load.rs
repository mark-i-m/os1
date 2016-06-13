//! The ELF loader

use core::mem::size_of;
use core::ptr;

use io::block::BlockDataBuffer;
use fs::ROOT_FS;
use machine;
use super::elf::*;

/// Exec the given file
/// TODO: use filenames, rather than inode numbers
/// TODO: pass arguments
/// Only returns if there is an error
pub fn exec(inode: usize) -> usize {
    // open the file
    let mut f = unsafe { (*ROOT_FS).open(inode).ok().unwrap() };

    // read the elf header
    let ehdr = unsafe {
        let mut buf = BlockDataBuffer::new(size_of::<Elf32Ehdr>());
        f.read(&mut buf);
        (*buf.get_ptr::<Elf32Ehdr>(0)).clone()
    };

    printf!("{:#?}\n", ehdr);

    // check the ELF header
    if ehdr.e_ident[EI_MAG0] != ELFMAG0 ||
       ehdr.e_ident[EI_MAG1] != ELFMAG1 ||
       ehdr.e_ident[EI_MAG2] != ELFMAG2 ||
       ehdr.e_ident[EI_MAG3] != ELFMAG3 ||
       ehdr.e_ident[EI_CLASS] != ELFCLASS32 ||
       ehdr.e_ident[EI_DATA] != ELFDATA2LSB ||
       ehdr.e_ident[EI_VERSION] != EV_CURRENT as u8 ||
       ehdr.e_type != ET_EXEC ||
       ehdr.e_machine != EM_386 ||
       ehdr.e_version != EV_CURRENT ||
       ehdr.e_phentsize != size_of::<Elf32Phdr>() as u16 {
            return 1;
       }

    // load the program header table
    f.seek(ehdr.e_phoff);

    let phdr_table = PhdrTable::new(ehdr.e_phnum as usize, {
        let mut buf = BlockDataBuffer::new(ehdr.e_phnum as usize * size_of::<Elf32Phdr>());
        f.read(&mut buf);
        buf
    });

    // load loadable segments into memory
    for phdr in phdr_table {
        if phdr.p_type == PT_LOAD {
            printf!("load segment at {:x} with size {:x} from file at {:X}\n", phdr.p_vaddr, phdr.p_memsz, phdr.p_offset);
            let mut buf = BlockDataBuffer::new(phdr.p_filesz);
            f.seek(phdr.p_offset);
            f.read(&mut buf);

            unsafe {
                ptr::copy(buf.get_ptr(0), phdr.p_vaddr as *mut u8, buf.size());
            }
        }
    }

    // set up the stack
    unsafe {
        *(0xFFFF_FFF0 as *mut u8) = 0;
    }

    printf!("entry {:X}\n", ehdr.e_entry);

    unsafe {
        machine::exec_jmp(ehdr.e_entry);
    }

    if ehdr.e_entry == 0 {
        exec_test(0);
    }

    return 0;
}

#[no_mangle]
#[inline(never)]
pub fn exec_test(x: usize) {
    panic!("Exec worked! {}", x);
}
