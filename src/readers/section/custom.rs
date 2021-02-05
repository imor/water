use crate::readers::binary::{BinaryReader, BinaryReaderError};
use crate::readers::binary::Result as BinaryReaderResult;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct CustomSectionReader<'a> {
    reader: BinaryReader<'a>,
    name: &'a str,
    data: &'a [u8],
}

#[derive(Debug)]
pub enum CustomReaderError {
    BinaryReaderError(BinaryReaderError),
}

impl From<BinaryReaderError> for CustomReaderError {
    fn from(e: BinaryReaderError) -> Self {
        CustomReaderError::BinaryReaderError(e)
    }
}

impl<'a> CustomSectionReader<'a> {
    pub(crate) fn new(buffer: &'a [u8]) -> BinaryReaderResult<CustomSectionReader<'a>> {
        let mut reader = BinaryReader::new(buffer);
        let name = reader.read_string()?;
        let data = &buffer[reader.get_position()..];
        Ok(CustomSectionReader { reader, name, data })
    }

    pub fn get_name(&self) -> &str {
        self.name
    }

    pub fn get_data(&self) -> &[u8] {
        self.data
    }
}
