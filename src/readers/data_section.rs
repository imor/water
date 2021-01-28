use crate::binary_reader::{BinaryReader, BinaryReaderError};
use crate::binary_reader::Result as BinaryReaderResult;
use std::result;
use crate::types::DataSegment;

#[derive(Eq, PartialEq, Debug)]
pub struct DataSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

#[derive(Debug)]
pub enum DataReaderError {
    BinaryReaderError(BinaryReaderError),
}

impl From<BinaryReaderError> for DataReaderError {
    fn from(e: BinaryReaderError) -> Self {
        DataReaderError::BinaryReaderError(e)
    }
}

pub type Result<T, E = DataReaderError> = result::Result<T, E>;

impl<'a> DataSectionReader<'a> {
    pub(crate) fn new(buffer: &'a [u8]) -> BinaryReaderResult<DataSectionReader<'a>> {
        let mut reader = BinaryReader::new(buffer);
        let count = reader.read_u32()?;
        Ok(DataSectionReader { reader, count })
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read(&mut self) -> Result<DataSegment> {
        Ok(self.reader.read_data_type()?)
    }
}
