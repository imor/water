use crate::readers::binary::{BinaryReader, BinaryReaderError};
use crate::readers::binary::Result as BinaryReaderResult;
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
        let num_labels = reader.read_leb128_u32()?;
        Ok(BranchTableReader { reader, num_labels })
    }

    pub fn skip_br_table(reader: &mut BinaryReader) -> BinaryReaderResult<(usize, usize)> {
        let start = reader.get_position();
        let num_labels = reader.read_leb128_u32()?;
        for _ in 0..num_labels {
            reader.read_leb128_u32()?;
        }
        reader.read_leb128_u32()?;
        let end = reader.get_position();
        Ok((start, end))
    }

    pub fn get_num_labels(&self) -> u32 {
        self.num_labels + 1
    }

    pub fn read(&mut self) -> Result<LabelIndex> {
        let label = self.reader.read_leb128_u32()?;
        Ok(LabelIndex(label))
    }
}

pub struct BranchTableLabelsIterator<'bin, 'btr> {
    reader: &'btr mut BranchTableReader<'bin>,
    error: bool,
    remaining_labels: u32
}

impl<'bin, 'btr> BranchTableLabelsIterator<'bin, 'btr> {
    fn new(reader: &'btr mut BranchTableReader<'bin>) -> BranchTableLabelsIterator<'bin, 'btr> {
        let remaining_labels = reader.get_num_labels();
        Self { reader, error: false, remaining_labels }
    }
}

impl<'bin, 'btr> Iterator for BranchTableLabelsIterator<'bin, 'btr> {
    type Item = Result<LabelIndex>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_labels == 0 || self.error {
            None
        } else {
            let result = self.reader.read();
            self.error = result.is_err();
            self.remaining_labels -= 1;
            Some(result)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.reader.get_num_labels() as usize;
        (count, Some(count))
    }
}

impl<'bin, 'btr> IntoIterator for &'btr mut BranchTableReader<'bin>
    where 'bin: 'btr
{
    type Item = Result<LabelIndex>;
    type IntoIter = BranchTableLabelsIterator<'bin, 'btr>;

    fn into_iter(self) -> Self::IntoIter {
        BranchTableLabelsIterator::new(self)
    }
}