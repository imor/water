use crate::readers::binary::{BinaryReader, BinaryReaderError};
use crate::readers::binary::Result as BinaryReaderResult;
use std::result;
use crate::types::GlobalType;

#[derive(Eq, PartialEq, Debug)]
pub struct GlobalSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

#[derive(Debug)]
pub enum GlobalReaderError {
    BinaryReaderError(BinaryReaderError),
}

impl From<BinaryReaderError> for GlobalReaderError {
    fn from(e: BinaryReaderError) -> Self {
        GlobalReaderError::BinaryReaderError(e)
    }
}

pub type Result<T, E = GlobalReaderError> = result::Result<T, E>;

impl<'a> GlobalSectionReader<'a> {
    pub(crate) fn new(buffer: &'a [u8]) -> BinaryReaderResult<GlobalSectionReader<'a>> {
        let mut reader = BinaryReader::new(buffer);
        let count = reader.read_u32()?;
        Ok(GlobalSectionReader { reader, count })
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read(&mut self) -> Result<GlobalType> {
        Ok(self.reader.read_global_type()?)
    }
}
