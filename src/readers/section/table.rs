use crate::readers::binary::{BinaryReader, BinaryReaderError};
use crate::readers::binary::Result as BinaryReaderResult;
use std::result;
use crate::types::TableType;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TableSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

#[derive(Debug)]
pub enum TableReaderError {
    BinaryReaderError(BinaryReaderError),
}

impl From<BinaryReaderError> for TableReaderError {
    fn from(e: BinaryReaderError) -> Self {
        TableReaderError::BinaryReaderError(e)
    }
}

pub type Result<T, E = TableReaderError> = result::Result<T, E>;

impl<'a> TableSectionReader<'a> {
    pub(crate) fn new(buffer: &'a [u8]) -> BinaryReaderResult<TableSectionReader<'a>> {
        let mut reader = BinaryReader::new(buffer);
        let count = reader.read_leb128_u32()?;
        Ok(TableSectionReader { reader, count })
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read(&mut self) -> Result<TableType> {
        Ok(self.reader.read_table_type()?)
    }
}