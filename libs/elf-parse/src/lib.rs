#![no_std]

use core::{mem::size_of, ptr};

const IDENT_SZ: usize = 16;

#[derive(PartialEq, Clone, Copy)]
pub struct ElfType(u16);
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
pub struct Elf64EHdr {
    pub ident: [u8; IDENT_SZ],
    pub file_type: ElfType,
    pub machine: MachineType,
    pub version: u32,
    pub entry: u64,
    pub ph_off: u64,
    pub sh_off: u64,
    pub flags: u32,
    pub eh_size: u16,
    pub ph_entsize: u16,
    pub ph_num: u16,
    pub sh_entsize: u16,
    pub sh_num: u16,
    pub sh_strndx: u16,
}
}

#[derive(Debug)]
pub enum Error {
    SizeTooSmall,
    InvalidMagic,
    UnsupportedFile,
    ParseError,
}

pub struct ElfFile<'a> {
    bytes: &'a [u8],
    pub hdr: Elf64EHdr,
}

impl<'b: 'a, 'a> ElfFile<'b> {
    pub fn new(bytes: &'b [u8]) -> Result<Self, Error> {
        let hdr_size = size_of::<Elf64EHdr>();
        if bytes.len() < hdr_size {
            return Err(Error::SizeTooSmall);
        }

        let file = ElfFile::<'b> {
            bytes,
            hdr: unsafe { ptr::read((&bytes[0..hdr_size]).as_ptr() as *const Elf64EHdr) },
        };

        if !file.verify_magic() {
            return Err(Error::InvalidMagic);
        }
        if file.hdr.file_type != ElfType::EXEC {
            return Err(Error::UnsupportedFile);
        }

        Ok(file)
    }

    fn verify_magic(&self) -> bool {
        self.hdr.ident[0] == 0x7f && self.hdr.ident[1..4].eq(b"ELF")
    }

    pub fn get_architecture(&self) -> MachineType {
        self.hdr.machine
    }
}
