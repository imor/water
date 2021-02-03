use std::result;
use crate::readers::preamble::PreambleReaderError::{BadVersion, BadMagicNumber};
use crate::readers::binary::{BinaryReader, BinaryReaderError};

const WASM_MAGIC_NUMBER: &[u8; 4] = b"\0asm";
const WASM_SUPPORTED_VERSION: u32 = 0x1;

#[derive(PartialEq, Eq, Debug)]
pub enum PreambleReaderError {
    BinaryReaderError(BinaryReaderError),
    BadVersion,
    BadMagicNumber,
}

impl From<BinaryReaderError> for PreambleReaderError {
    fn from(e: BinaryReaderError) -> Self {
        PreambleReaderError::BinaryReaderError(e)
    }
}

pub type Result<T, E = PreambleReaderError> = result::Result<T, E>;

#[derive(Eq, PartialEq, Debug)]
pub struct PreambleReader<'a> {
    reader: BinaryReader<'a>,
}

impl<'a> PreambleReader<'a> {
    pub fn new(buffer: &[u8]) -> PreambleReader {
        let reader = BinaryReader::new(buffer);
        PreambleReader { reader }
    }

    pub fn read_preamble(&mut self) -> Result<(usize, u32)> {
        let magic_number = self.reader.read_bytes(4)?;
        if magic_number == WASM_MAGIC_NUMBER {
            let version = self.reader.read_double_word()?;
            if version == WASM_SUPPORTED_VERSION {
                Ok((self.reader.get_position(), version))
            } else {
                Err(BadVersion)
            }
        } else {
            Err(BadMagicNumber)
        }
    }
}
