#![no_std]

use core::{mem::size_of, ptr};

const IDENT_SZ: usize = 16;

#[derive(PartialEq, Clone, Copy)]
struct ElfType(u16);
impl ElfType {
    const NONE: ElfType = ElfType(0);
    const REL: ElfType = ElfType(1);
    const EXEC: ElfType = ElfType(2);
    const DYN: ElfType = ElfType(3);
    const CORE: ElfType = ElfType(4);
}

#[derive(PartialEq, Clone, Copy)]
pub struct MachineType(u16);
impl MachineType {
    pub const AARCH64: MachineType = MachineType(183);
    pub const X86_64: MachineType = MachineType(62);
}

#[repr(C)]
struct Elf64EHdr {
    ident: [u8; IDENT_SZ],
    file_type: ElfType,
    machine: MachineType,
    version: u32,
    entry: u64,
    ph_off: u64,
    sh_off: u64,
    flags: u32,
    eh_size: u16,
    ph_entsize: u16,
    ph_num: u16,
    sh_entsize: u16,
    sh_num: u16,
    sh_strndx: u16,
}

#[derive(Debug)]
pub enum Error {
    SizeTooSmall,
    InvalidMagic,
    UnsupportedFile,
    ParseError,
}

pub struct ElfFile {
    pub entry: Option<u64>,

    hdr: Elf64EHdr,
}

impl ElfFile {
    pub fn new(bytes: &[u8]) -> Result<Self, Error> {
        let hdr_size = size_of::<Elf64EHdr>();
        if bytes.len() < hdr_size {
            return Err(Error::SizeTooSmall);
        }

        let mut file = ElfFile {
            entry: None,
            hdr: unsafe { ptr::read((&bytes[0..hdr_size]).as_ptr() as *const Elf64EHdr) },
        };

        if !file.verify_magic() {
            return Err(Error::InvalidMagic);
        }
        if file.hdr.file_type != ElfType::EXEC {
            return Err(Error::UnsupportedFile);
        }

        file.entry = match file.hdr.entry {
            0 => None,
            addr => Some(addr),
        };

        Ok(file)
    }

    fn verify_magic(&self) -> bool {
        self.hdr.ident[0] == 0x7f && self.hdr.ident[1..4].eq(b"ELF")
    }

    pub fn get_architecture(&self) -> MachineType {
        self.hdr.machine
    }
}
