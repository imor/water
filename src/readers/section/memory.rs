use crate::readers::binary::{BinaryReader, BinaryReaderError};
use crate::readers::binary::Result as BinaryReaderResult;
use std::result;
use crate::types::MemoryType;
use crate::readers::common::{SectionReader, SectionItemIterator};

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct MemorySectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

#[derive(Debug, PartialEq, Eq)]
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

impl<'a> SectionReader for MemorySectionReader<'a> {
    type Item = MemoryType;
    type Error = MemoryReaderError;

    fn read(&mut self) -> Result<Self::Item, Self::Error> {
        self.read()
    }

    fn get_count(&self) -> u32 {
        self.get_count()
    }
}

impl<'a> IntoIterator for MemorySectionReader<'a> {
    type Item = Result<MemoryType>;
    type IntoIter = SectionItemIterator<MemorySectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionItemIterator::new(self)
    }
}
