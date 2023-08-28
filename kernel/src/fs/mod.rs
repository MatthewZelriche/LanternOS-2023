use alloc::vec::Vec;
use fatfs::Seek;

use crate::peripherals::EMMC2;

#[derive(Clone, Copy)]
pub struct Fat32FileSystem {
    sector: u32,
    offset: u32,
}

impl Fat32FileSystem {
    pub fn new() -> Self {
        Fat32FileSystem {
            sector: 0,
            offset: 0,
        }
    }
}

impl fatfs::IoBase for Fat32FileSystem {
    type Error = ();
}

impl fatfs::Read for Fat32FileSystem {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let byte_count = buf.len();
        let offset = byte_count % 512;
        let num_sectors = (byte_count / 512) + if offset != 0 { 1 } else { 0 };
        let sector_bytes = num_sectors * 512;

        let mut vec = Vec::with_capacity(sector_bytes);
        unsafe {
            // Sound because we confirm sector_bytes is less than capacity, and
            // because its a vector of u8 and u8 is always considered "initialized"
            // regardless of the values we end up with
            assert!(sector_bytes <= vec.capacity());
            vec.set_len(sector_bytes);
        }

        EMMC2.get().unwrap().lock().emmc_transfer_blocks(
            self.sector,
            num_sectors as u32,
            vec.as_mut_slice(),
            false,
        );

        let byte_slice = &vec[self.offset as usize..];
        let final_len = buf.len().min(byte_slice.len());
        let byte_slice = &byte_slice[..final_len];
        buf.copy_from_slice(byte_slice);

        self.seek(fatfs::SeekFrom::Current(final_len as i64))?;
        Ok(final_len)
    }
}

impl fatfs::Write for Fat32FileSystem {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let byte_count = buf.len();
        // Read in the first block
        let mut vec = Vec::with_capacity(512);
        unsafe {
            // Sound because we confirm sector_bytes is less than capacity, and
            // because its a vector of u8 and u8 is always considered "initialized"
            // regardless of the values we end up with
            assert!(512 <= vec.capacity());
            vec.set_len(512);
        }
        EMMC2
            .get()
            .unwrap()
            .lock()
            .emmc_transfer_blocks(self.sector, 1, vec.as_mut_slice(), false);
        // Write into the first sector
        let bytes_left_in_first_sector = 512 - self.offset;
        let bytes_to_write_first_sector = bytes_left_in_first_sector.min(byte_count as u32);
        let slice = &mut vec
            [self.offset as usize..self.offset as usize + bytes_to_write_first_sector as usize];
        slice.copy_from_slice(&buf[..bytes_to_write_first_sector as usize]);
        EMMC2
            .get()
            .unwrap()
            .lock()
            .emmc_transfer_blocks(self.sector, 1, vec.as_mut_slice(), true);

        self.seek(fatfs::SeekFrom::Current(bytes_to_write_first_sector as i64))?;

        if byte_count > bytes_to_write_first_sector as usize {
            // Still need to handle writes across multiple sectors, somewhat more complicated
            todo!()
        } else {
            Ok(bytes_to_write_first_sector as usize)
        }
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        // No flushing necessary
        Ok(())
    }
}

impl fatfs::Seek for Fat32FileSystem {
    fn seek(&mut self, pos: fatfs::SeekFrom) -> Result<u64, Self::Error> {
        match pos {
            fatfs::SeekFrom::Start(i) => {
                self.sector = i as u32 / 512;
                self.offset = i as u32 % 512;
                Ok(i)
            }
            fatfs::SeekFrom::End(_) => {
                // Currently not supported
                unimplemented!()
            }
            fatfs::SeekFrom::Current(i) => {
                let curr = (self.sector * 512) + self.offset;
                let new_pos = curr as i64 + i;
                self.seek(fatfs::SeekFrom::Start(new_pos as u64))?;
                Ok(new_pos as u64)
            }
        }
    }
}
