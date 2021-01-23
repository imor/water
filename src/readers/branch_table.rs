use crate::binary_reader::{BinaryReader, BinaryReaderError};
use crate::binary_reader::Result as BinaryReaderResult;
use std::result;
use crate::types::LabelIndex;

#[derive(Eq, PartialEq, Debug)]
pub struct BranchTableReader<'a> {
    reader: BinaryReader<'a>,
    num_labels: u32,
}

#[derive(Debug)]
pub enum BranchReaderError {
    BinaryReaderError(BinaryReaderError),
}

impl From<BinaryReaderError> for BranchReaderError {
    fn from(e: BinaryReaderError) -> Self {
        BranchReaderError::BinaryReaderError(e)
    }
}

pub type Result<T, E = BranchReaderError> = result::Result<T, E>;

impl<'a> BranchTableReader<'a> {
    pub(crate) fn new(buffer: &'a [u8]) -> BinaryReaderResult<BranchTableReader<'a>> {
        let mut reader = BinaryReader::new(buffer);
        let num_labels = reader.read_var_u32()?;
        Ok(BranchTableReader { reader, num_labels })
    }

    pub fn get_num_labels(&self) -> u32 {
        self.num_labels + 1
    }

    pub fn read(&mut self) -> Result<LabelIndex> {
        let label = self.reader.read_var_u32()?;
        Ok(LabelIndex(label))
    }
}
