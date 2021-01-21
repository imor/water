use crate::binary_reader::{BinaryReader, BinaryReaderError};
use crate::binary_reader::Result as BinaryReaderResult;
use std::result;

#[derive(Eq, PartialEq, Debug)]
pub struct FunctionSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

#[derive(Debug)]
pub enum FunctionReaderError {
    BinaryReaderError(BinaryReaderError),
}

impl From<BinaryReaderError> for FunctionReaderError {
    fn from(e: BinaryReaderError) -> Self {
        FunctionReaderError::BinaryReaderError(e)
    }
}

pub type Result<T, E = FunctionReaderError> = result::Result<T, E>;

impl<'a> FunctionSectionReader<'a> {
    pub(crate) fn new(buffer: &'a [u8]) -> BinaryReaderResult<FunctionSectionReader<'a>> {
        let mut reader = BinaryReader::new(buffer);
        let count = reader.read_var_u32()?;
        Ok(FunctionSectionReader { reader, count })
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read(&mut self) -> Result<u32> {
        Ok(self.reader.read_var_u32()?)
    }
}