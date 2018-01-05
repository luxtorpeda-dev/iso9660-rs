#![feature(untagged_unions)]
#![feature(repr_align)]
#![feature(attr_literals)]

#[macro_use]
extern crate static_assertions;

use std::io::{Result, SeekFrom, Read, Seek, Error, ErrorKind};
use std::fs::File;
use std::path::Path;

use volume_descriptor::VolumeDescriptor;

mod both_endian;
mod volume_descriptor;


#[repr(C)]
union Block {
    // CDROMs contain 2048 byte blocks
    bytes: [u8; 2048],
    volume_descriptor: VolumeDescriptor
}

pub struct ISO9660 {
    file: File,
    block: Block
}

impl ISO9660 {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<ISO9660> {
        let mut fs = ISO9660 {
            file: File::open(&path)?,
            block: Block { bytes: [0; 2048] }
        };

        // Skip the "system area"
        fs.file.seek(SeekFrom::Start(16 * 2048))?;

        // Read volume descriptors
        loop {
            fs.file.read(unsafe { &mut fs.block.bytes })?;
            let desc = unsafe { &fs.block.volume_descriptor };

            if &desc.identifier != b"CD001" || desc.version != 1 {
                // XXX Change error type
                return Err(Error::new(ErrorKind::Other, "Not ISO9660"))
            }

            match desc.type_code {
                // Boot record
                0 => {}
                // Primary volume descriptor
                1 => {}
                // Supplementary volume descriptor
                2 => {}
                // Volume partition descriptor
                3 => {}
                // Volume descriptor set terminator
                255 => { break; }
                _ => {}
            }
        }

        Ok(fs)
    }

    /// Read the block at a given LBA (logical block address)
    fn read_block(&mut self, lba: u64) -> Result<&Block> {
        self.file.seek(SeekFrom::Start(lba * 2048))?;
        self.file.read(unsafe { &mut self.block.bytes })?;
        Ok(&self.block)
    }
}

assert_eq_size!(block_size_eq; Block, [u8; 2048]);
