use crate::readers::binary::{BinaryReader, BinaryReaderError};
use crate::readers::binary::Result as BinaryReaderResult;
use std::result;
use crate::types::{ElementSegment, TableIndex, FuncIndex};
use crate::readers::common::{SectionReader, SectionItemIterator};

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ElementSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

#[derive(Debug, Eq, PartialEq)]
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
        let count = reader.read_leb128_u32()?;
        Ok(ElementSectionReader { reader, count })
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read<'b>(&mut self) -> Result<ElementSegment<'b>>
        where 'a: 'b
    {
        Ok(self.read_element_segment()?)
    }

    fn read_element_segment<'b>(&mut self) -> Result<ElementSegment<'b>>
        where 'a: 'b
    {
        let table_index = TableIndex(self.reader.read_leb128_u32()?);
        let instruction_reader = self.reader.create_instruction_reader()?;
        let len = self.reader.read_leb128_u32()?;
        let mut func_indices = Vec::with_capacity(len as usize);
        for _ in 0..len {
            let func_index = FuncIndex(self.reader.read_leb128_u32()?);
            func_indices.push(func_index);
        }

        Ok(ElementSegment { table_index, instruction_reader, function_indices: func_indices.into_boxed_slice() })
    }
}

impl<'a> SectionReader for ElementSectionReader<'a> {
    type Item = ElementSegment<'a>;
    type Error = ElementReaderError;

    fn read(&mut self) -> Result<Self::Item, Self::Error> {
        self.read()
    }

    fn get_count(&self) -> u32 {
        self.get_count()
    }
}

impl<'a> IntoIterator for ElementSectionReader<'a> {
    type Item = Result<ElementSegment<'a>>;
    type IntoIter = SectionItemIterator<ElementSectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionItemIterator::new(self)
    }
}
