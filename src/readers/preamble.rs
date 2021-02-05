use std::result;
use crate::readers::binary::{BinaryReader, BinaryReaderError};
use std::convert::TryFrom;

#[derive(PartialEq, Eq, Debug)]
pub enum PreambleReaderError {
    BinaryReaderError(BinaryReaderError),
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

    pub fn read_preamble(&mut self) -> Result<(usize, &'a [u8; 4], u32)> {
        let magic_number = self.reader.read_bytes(4)?;
        let version = self.reader.read_double_word()?;
        Ok((self.reader.get_position(), <&[u8; 4]>::try_from(magic_number).unwrap(), version))
    }
}
