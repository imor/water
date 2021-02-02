use crate::readers::binary::{BinaryReader, BinaryReaderError};
use crate::readers::binary::Result as BinaryReaderResult;
use std::result;
use crate::types::ElementSegment;

#[derive(Eq, PartialEq, Debug)]
pub struct ElementSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

#[derive(Debug)]
pub enum ElementReaderError {
    BinaryReaderError(BinaryReaderError),
}

impl From<BinaryReaderError> for ElementReaderError {
    fn from(e: BinaryReaderError) -> Self {
        ElementReaderError::BinaryReaderError(e)
    }
}

pub type Result<T, E = ElementReaderError> = result::Result<T, E>;

impl<'a> ElementSectionReader<'a> {
    pub(crate) fn new(buffer: &'a [u8]) -> BinaryReaderResult<ElementSectionReader<'a>> {
        let mut reader = BinaryReader::new(buffer);
        let count = reader.read_u32()?;
        Ok(ElementSectionReader { reader, count })
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read(&mut self) -> Result<ElementSegment> {
        Ok(self.reader.read_element_type()?)
    }
}
