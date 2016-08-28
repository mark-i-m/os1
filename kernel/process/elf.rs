//! Elf data type definitions
//!
//! Based on http://flint.cs.yale.edu/cs422/doc/ELF_Format.pdf
//!
//! It only contains definitions sufficient to load an executable ELF.

#![allow(dead_code)]

use io::block::BlockDataBuffer;

/// Unsigned program address
pub type Elf32Addr = usize;
/// Unsigned medium integer
pub type Elf32Half = u16;
/// Unsinged file offset
pub type Elf32Off = usize;
/// Signed large integer
pub type Elf32Sword = isize;
/// Unsigned large interger
pub type Elf32Word = usize;

/// The ELF header of an ELF file
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Elf32Ehdr {
    /// The initial bytes of the file mark it as an ELF file, give the type,
    /// and provide information about how to interpret the contents.
    pub e_ident: [u8; EI_NIDENT],
    /// What type of ELF file is this?
    pub e_type: Elf32Half,
    /// What type of machine is the ELF designed for?
    pub e_machine: Elf32Half,
    /// What version of the ELF spec?
    pub e_version: Elf32Word,
    /// The virtual address of the first instruction of the code to run
    pub e_entry: Elf32Addr,
    /// The program header table's file offset in bytes
    pub e_phoff: Elf32Off,
    /// The section header table's file offset in bytes
    pub e_shoff: Elf32Off,
    /// Processor-specific flags
    pub e_flags: Elf32Word,
    /// Size of the ELF header
    pub e_ehsize: Elf32Half,
    /// Size of an entry in the program header table
    pub e_phentsize: Elf32Half,
    /// Number of program headers
    pub e_phnum: Elf32Half,
    /// Size of an entry in the section header table
    pub e_shentsize: Elf32Half,
    /// Number of section headers
    pub e_shnum: Elf32Half,
    /// The section header table index of the entry associated with the
    /// section name string table.
    pub e_shstrndx: Elf32Half,
}

// Possible values for e_type. The e_type is still defined
// as a Elf32Half above, though, for compatibility. The same
// holds for the enums defined in the rest of this module.

/// No file type
pub const ET_NONE: u16 = 0;
/// Relocatable file
pub const ET_REL: u16 = 1;
/// Executable file
pub const ET_EXEC: u16 = 2;
/// Shared object file
pub const ET_DYN: u16 = 3;
/// Corefile
pub const ET_CORE: u16 = 4;
/// Processor-specific
pub const ET_LOPROC: u16 = 0xff00;
/// Processor-specific
pub const ET_HIPROC: u16 = 0xffff;

// Possible values for e_machine
/// No machine
pub const EM_NONE: u16 = 0;
/// AT&T WE 32100
pub const EM_M32: u16 = 1;
/// SPARC
pub const EM_SPARC: u16 = 2;
/// Intel 80386
pub const EM_386: u16 = 3;
/// Motorola 68000
pub const EM_68K: u16 = 4;
/// Motorola 88000
pub const EM_88K: u16 = 5;
/// Intel 80860
pub const EM_860: u16 = 6;
/// MIPS RS3000
pub const EM_MIPS: u16 = 7;

// Possible value for e_version
/// Invalid version
pub const EV_NONE: usize = 0;
/// Current version
pub const EV_CURRENT: usize = 1;

/// Size of e_ident
pub const EI_NIDENT: usize = 16;

// Indices of e_ident
/// Magic byte 0
pub const EI_MAG0: usize = 0;
/// Magic byte 1
pub const EI_MAG1: usize = 1;
/// Magic byte 2
pub const EI_MAG2: usize = 2;
/// Magic byte 3
pub const EI_MAG3: usize = 3;
/// File class
pub const EI_CLASS: usize = 4;
/// Data encoding
pub const EI_DATA: usize = 5;
/// File version
pub const EI_VERSION: usize = 6;
/// Start of padding bytes
pub const EI_PAD: usize = 7;

// Magic bytes
pub const ELFMAG0: u8 = 0x7f;
pub const ELFMAG1: u8 = 'E' as u8;
pub const ELFMAG2: u8 = 'L' as u8;
pub const ELFMAG3: u8 = 'F' as u8;

// Possible file classes
/// Invalid class
pub const ELFCLASSNONE: u8 = 0;
/// 32-bit objects
pub const ELFCLASS32: u8 = 1;
/// 64-bit objects
pub const ELFCLASS64: u8 = 2;

// Possible data encodings
/// Invalid encoding
pub const ELFDATANONE: u8 = 0;
/// 2's complement, little endian
pub const ELFDATA2LSB: u8 = 1;
/// 2's complement, big endian
pub const ELFDATA2MSB: u8 = 2;

/// A single program header in the program header table
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Elf32Phdr {
    /// What kind of segment is this?
    pub p_type: Elf32Word,
    /// The segment's file offset in bytes
    pub p_offset: Elf32Off,
    /// The virtual address at which the first byte of the segment goes in memory
    pub p_vaddr: Elf32Addr,
    /// Allows the linker to request a physical address
    pub p_paddr: Elf32Addr,
    /// Number of bytes in the file image of the segment
    pub p_filesz: Elf32Word,
    /// Number of bytes in the memory image of the segment
    pub p_memsz: Elf32Word,
    /// Flags for the segment
    pub p_flags: Elf32Word,
    /// Request alignment for the segment
    pub p_align: Elf32Word,
}

// Possible values of p_type
/// Null (unused) header
pub const PT_NULL: usize = 0;
/// Loadable segment
pub const PT_LOAD: usize = 1;
/// Dynamic linking information
pub const PT_DYNAMIC: usize = 2;
/// Specifies path to interpreter
pub const PT_INTERP: usize = 3;
/// Specifies location of auxiliary information
pub const PT_NOTE: usize = 4;
/// Reserved but has unknown semantics
pub const PT_SHLIB: usize = 5;
/// The entry specifies the location of the Program header table itself
pub const PT_PHDR: usize = 6;
/// PT_LOPROC through PT_HIPROC is an inclusive range reserved for processor
/// specific semantics
pub const PT_LOPROC: usize = 0x7000_0000;
pub const PT_HIPROC: usize = 0x7fff_ffff;

/// A safe wrapper around a pointer to a Phdr table
pub struct PhdrTable {
    phdr_table: BlockDataBuffer,
    phnum: usize,
    which: usize,
}

impl PhdrTable {
    pub fn new(phnum: usize, bdb: BlockDataBuffer) -> PhdrTable {
        PhdrTable {
            phdr_table: bdb,
            phnum: phnum,
            which: 0,
        }
    }
}

impl Iterator for PhdrTable {
    type Item = Elf32Phdr;

    fn next(&mut self) -> Option<Elf32Phdr> {
        if self.which < self.phnum {
            self.which += 1;
            unsafe { Some(*self.phdr_table.get_ptr::<Elf32Phdr>(self.which - 1)) }
        } else {
            None
        }
    }
}
