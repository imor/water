use crate::readers::binary::{BinaryReader, BinaryReaderError};
use crate::readers::binary::Result as BinaryReaderResult;
use std::result;
use crate::types::MemoryType;

#[derive(Eq, PartialEq, Debug)]
pub struct MemorySectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

#[derive(Debug)]
pub enum MemoryReaderError {
    BinaryReaderError(BinaryReaderError),
}

impl From<BinaryReaderError> for MemoryReaderError {
    fn from(e: BinaryReaderError) -> Self {
        MemoryReaderError::BinaryReaderError(e)
    }
}

pub type Result<T, E = MemoryReaderError> = result::Result<T, E>;

impl<'a> MemorySectionReader<'a> {
    pub(crate) fn new(buffer: &'a [u8]) -> BinaryReaderResult<MemorySectionReader<'a>> {
        let mut reader = BinaryReader::new(buffer);
        let count = reader.read_leb128_u32()?;
        Ok(MemorySectionReader { reader, count })
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read(&mut self) -> Result<MemoryType> {
        Ok(self.reader.read_memory_type()?)
    }
}
