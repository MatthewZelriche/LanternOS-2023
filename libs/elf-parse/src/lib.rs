#![no_std]

use core::{mem::size_of, ptr};

const IDENT_SZ: usize = 16;

#[repr(transparent)]
struct ElfType(u16);
impl ElfType {
    const NONE: u16 = 0;
    const REL: u16 = 1;
    const EXEC: u16 = 2;
    const DYN: u16 = 3;
    const CORE: u16 = 4;
}

#[repr(C)]
#[repr(packed)]
struct Elf64EHdr {
    ident: [u8; IDENT_SZ],
    r#type: u16,
    machine: u16,
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
    ParseError,
}

pub struct ElfFile {
    hdr: Elf64EHdr,
}

impl ElfFile {
    pub fn new(bytes: &[u8]) -> Result<Self, Error> {
        let hdr_size = size_of::<Elf64EHdr>();
        if bytes.len() < hdr_size {
            return Err(Error::SizeTooSmall);
        }

        let file = ElfFile {
            hdr: unsafe { ptr::read((&bytes[0..hdr_size]).as_ptr() as *const Elf64EHdr) },
        };

        Ok(file)
    }
}
