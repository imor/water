use std::result;
use std::convert::TryInto;
use crate::binary_reader::BinaryReaderError::{UnexpectedEof, BadVersion, BadMagicNumber};

const WASM_MAGIC_NUMBER: &[u8; 4] = b"\0asm";
const WASM_SUPPORTED_VERSION: u32 = 0x1;

#[derive(PartialEq, Eq, Debug)]
pub enum BinaryReaderError {
    UnexpectedEof,
    BadVersion,
    BadMagicNumber,
}

pub type Result<T, E = BinaryReaderError> = result::Result<T, E>;

pub struct BinaryReader<'a> {
    buffer: &'a [u8],
    position: usize,
}

impl<'a> BinaryReader<'a> {
    pub fn new(buffer: &[u8]) -> BinaryReader {
        BinaryReader {
            buffer,
            position: 0,
        }
    }

    fn ensure_has_bytes(&self, n: usize) -> Result<()> {
        if self.position + n <= self.buffer.len() {
            Ok(())
        } else {
            Err(UnexpectedEof)
        }
    }

    fn read_bytes(&mut self, n: usize) -> Result<&'a [u8]> {
        self.ensure_has_bytes(n)?;
        let start = self.position;
        self.position += n;
        Ok(&self.buffer[start..self.position])
    }

    fn read_u32(&mut self) -> Result<u32> {
        self.ensure_has_bytes(4)?;
        let word = u32::from_le_bytes(
            self.buffer[self.position..self.position + 4]
                .try_into().unwrap()
        );
        self.position += 4;
        Ok(word)
    }

    pub fn read_file_header(&mut self) -> Result<(usize, u32)> {
        let magic_number = self.read_bytes(4)?;
        if magic_number == WASM_MAGIC_NUMBER {
            let version = self.read_u32()?;
            if version == WASM_SUPPORTED_VERSION {
                Ok((self.position, version))
            } else {
                Err(BadVersion)
            }
        } else {
            Err(BadMagicNumber)
        }
    }
}