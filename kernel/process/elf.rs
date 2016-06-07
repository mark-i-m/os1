//! Elf data type definitions
//!
//! Based on http://flint.cs.yale.edu/cs422/doc/ELF_Format.pdf
//!
//! It only contains definitions sufficient to load an executable ELF.

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
pub struct Elf32Ehdr {
    /// The initial bytes of the file mark it as an ELF file, give the type,
    /// and provide information about how to interpret the contents.
    e_ident:        [u8; EI_NIDENT],
    /// What type of ELF file is this?
    e_type:         Elf32Half,
    /// What type of machine is the ELF designed for?
    e_machine:      Elf32Half,
    /// What version of the ELF spec?
    e_version:      Elf32Word,
    /// The virtual address of the first instruction of the code to run
    e_entry:        Elf32Addr,
    /// The program header table's file offset in bytes
    e_phoff:        Elf32Off,
    /// The section header table's file offset in bytes
    e_shoff:        Elf32Off,
    /// Processor-specific flags
    e_flags:        Elf32Word,
    /// Size of the ELF header
    e_ehsize:       Elf32Half,
    /// Size of an entry in the program header table
    e_phentsize:    Elf32Half,
    /// Number of program headers
    e_phnum:        Elf32Half,
    /// Size of an entry in the section header table
    e_shentsize:    Elf32Half,
    /// Number of section headers
    e_shnum:        Elf32Half,
    /// The section header table index of the entry associated with the
    /// section name string table.
    e_shstrndx:     Elf32Half,
}

/// Possible values for e_type. The e_type is still defined
/// as a Elf32Half above, though, for compatibility. The same
/// holds for the enums defined in the rest of this module.
pub enum EType {
    /// No file type
    ET_NONE = 0,
    /// Relocatable file
    ET_REL = 1,
    /// Executable file
    ET_EXEC = 2,
    /// Shared object file
    ET_DYN = 3,
    /// Corefile
    ET_CORE = 4,
    /// Processor-specific
    ET_LOPROC = 0xff00,
    /// Processor-specific
    ET_HIPROC = 0xffff,
}

/// Possible values for e_machine
pub enum EMachine {
    /// No machine
    EM_NONE = 0,
    /// AT&T WE 32100
    EM_M32 = 1,
    /// SPARC
    EM_SPARC = 2,
    /// Intel 80386
    EM_386 = 3,
    /// Motorola 68000
    EM_68K = 4,
    /// Motorola 88000
    EM_88K = 5,
    /// Intel 80860
    EM_860 = 6,
    /// MIPS RS3000
    EM_MIPS = 7,
}

/// Possible value for e_version
pub enum EVersion {
    /// Invalid version
    EV_NONE = 0,
    /// Current version
    EV_CURRENT = 1,
}

/// Size of e_ident
pub const EI_NIDENT: usize = 16;

/// Indices of e_ident
pub enum EIdent {
    /// Magic byte 0
    EI_MAG0 = 0,
    /// Magic byte 1
    EI_MAG1 = 1,
    /// Magic byte 2
    EI_MAG2 = 2,
    /// Magic byte 3
    EI_MAG3 = 3,
    /// File class
    EI_CLASS = 4,
    /// Data encoding
    EI_DATA = 5,
    /// File version
    EI_VERSION = 6,
    /// Start of padding bytes
    EI_PAD = 7,
}

/// Magic bytes
pub enum EMagic {
    ELFMAG0 = 0x7f,
    ELFMAG1 = 'E',
    ELFMAG2 = 'L',
    ELFMAG3 = 'F',
}

/// Possible file classes
pub enum EClass {
    /// Invalid class
    ELFCLASSNONE = 0,
    /// 32-bit objects
    ELFCLASS32 = 1,
    /// 64-bit objects
    ELFCLASS64 = 2,
}

/// Possible data encodings
pub enum EData {
    /// Invalid encoding
    ELFDATANONE = 0,
    /// 2's complement, little endian
    ELFDATA2LSB = 1,
    /// 2's complement, big endian
    ELFDATA2MSB = 2,
}

/// A single program header in the program header table
pub struct Elf32Phdr{
    /// What kind of segment is this?
    p_type:     Elf32Word,
    /// The segment's file offset in bytes
    p_offset:   Elf32Off,
    /// The virtual address at which the first byte of the segment goes in memory
    p_vaddr:    Elf32Addr,
    /// Allows the linker to request a physical address
    p_paddr:    Elf32Addr,
    /// Number of bytes in the file image of the segment
    p_filesz:   Elf32Word,
    /// Number of bytes in the memory image of the segment
    p_memsz:    Elf32Word,
    /// Flags for the segment
    p_flags:    Elf32Word,
    /// Request alignment for the segment
    p_align:    Elf32Word,
}

/// Possible values of p_type
pub enum PType {
    /// Null (unused) header
    PT_NULL = 0,
    /// Loadable segment
    PT_LOAD = 1,
    /// Dynamic linking information
    PT_DYNAMIC = 2,
    /// Specifies path to interpreter
    PT_INTERP = 3,
    /// Specifies location of auxiliary information
    PT_NOTE = 4,
    /// Reserved but has unknown semantics
    PT_SHLIB = 5,
    /// The entry specifies the location of the Program header table itself
    PT_PHDR = 6,
    /// PT_LOPROC through PT_HIPROC is an inclusive range reserved for processor
    /// specific semantics
    PT_LOPROC = 0x7000_0000,
    PT_HIPROC = 0x7fff_ffff,
}
