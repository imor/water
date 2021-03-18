use crate::readers::binary::{BinaryReader, BinaryReaderError};
use crate::readers::binary::Result as BinaryReaderResult;
use std::result;
use crate::types::GlobalSegment;
use crate::readers::common::{SectionReader, SectionItemIterator};

#[derive(Clone, Eq, PartialEq, Debug)]
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
        let count = reader.read_leb128_u32()?;
        Ok(GlobalSectionReader { reader, count })
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read<'b>(&mut self) -> Result<GlobalSegment<'b>>
        where 'a: 'b
    {
        self.read_global_segment()
    }

    fn read_global_segment<'b>(&mut self) -> Result<GlobalSegment<'b>>
        where 'a: 'b
    {
        let global_type = self.reader.read_global_type()?;
        let instruction_reader = self.reader.create_instruction_reader()?;
        Ok(GlobalSegment { global_type, instruction_reader })
    }
}

impl<'a> SectionReader for GlobalSectionReader<'a> {
    type Item = GlobalSegment<'a>;
    type Error = GlobalReaderError;

    fn read(&mut self) -> Result<Self::Item, Self::Error> {
        self.read()
    }

    fn get_count(&self) -> u32 {
        self.get_count()
    }
}

impl<'a> IntoIterator for GlobalSectionReader<'a> {
    type Item = Result<GlobalSegment<'a>>;
    type IntoIter = SectionItemIterator<GlobalSectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionItemIterator::new(self)
    }
}
