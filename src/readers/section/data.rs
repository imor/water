use crate::readers::binary::{BinaryReader, BinaryReaderError};
use crate::readers::binary::Result as BinaryReaderResult;
use std::result;
use crate::types::{DataSegment, MemoryIndex};
use crate::readers::common::{SectionReader, SectionItemIterator};

#[derive(Clone, Eq, PartialEq, Debug)]
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
        let count = reader.read_leb128_u32()?;
        Ok(DataSectionReader { reader, count })
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read<'b>(&mut self) -> Result<DataSegment<'b>>
        where 'a: 'b
    {
        self.read_data_segment()
    }

    fn read_data_segment<'b>(&mut self) -> Result<DataSegment<'b>>
        where 'a: 'b
    {
        let memory_index = MemoryIndex(self.reader.read_leb128_u32()?);
        let instruction_reader = self.reader.create_instruction_reader()?;
        let bytes = self.reader.read_bytes_vec()?;
        Ok(DataSegment { memory_index, instruction_reader, bytes })
    }
}

impl<'a> SectionReader for DataSectionReader<'a> {
    type Item = DataSegment<'a>;
    type Error = DataReaderError;

    fn read(&mut self) -> Result<Self::Item, Self::Error> {
        self.read()
    }

    fn get_count(&self) -> u32 {
        self.get_count()
    }
}

impl<'a> IntoIterator for DataSectionReader<'a> {
    type Item = Result<DataSegment<'a>>;
    type IntoIter = SectionItemIterator<DataSectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionItemIterator::new(self)
    }
}
