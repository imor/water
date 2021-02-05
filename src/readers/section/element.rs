use crate::readers::binary::{BinaryReader, BinaryReaderError};
use crate::readers::binary::Result as BinaryReaderResult;
use std::result;
use crate::types::{ElementSegment, TableIndex, FuncIndex};

#[derive(Clone, Eq, PartialEq, Debug)]
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
        let count = reader.read_leb128_u32()?;
        Ok(ElementSectionReader { reader, count })
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read(&mut self) -> Result<ElementSegment> {
        Ok(self.read_element_segment()?)
    }

    fn read_element_segment(&mut self) -> Result<ElementSegment> {
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
