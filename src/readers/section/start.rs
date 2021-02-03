use crate::readers::binary::{BinaryReader, BinaryReaderError};
use crate::readers::binary::Result as BinaryReaderResult;
use crate::types::FuncIndex;

#[derive(Eq, PartialEq, Debug)]
pub struct StartSectionReader<'a> {
    reader: BinaryReader<'a>,
    func_index: FuncIndex,
}

#[derive(Debug)]
pub enum StartReaderError {
    BinaryReaderError(BinaryReaderError),
}

impl From<BinaryReaderError> for StartReaderError {
    fn from(e: BinaryReaderError) -> Self {
        StartReaderError::BinaryReaderError(e)
    }
}

impl<'a> StartSectionReader<'a> {
    pub(crate) fn new(buffer: &'a [u8]) -> BinaryReaderResult<StartSectionReader<'a>> {
        let mut reader = BinaryReader::new(buffer);
        let index = reader.read_leb128_u32()?;
        Ok(StartSectionReader { reader, func_index: FuncIndex(index)})
    }

    pub fn get_func_index(&self) -> FuncIndex {
        self.func_index
    }
}
