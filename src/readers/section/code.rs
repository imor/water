use crate::readers::binary::{BinaryReader, BinaryReaderError};
use crate::readers::binary::Result as BinaryReaderResult;
use std::result;
use crate::types::Locals;
use crate::InstructionReader;
use crate::readers::common::{SectionReader, SectionItemIterator};

pub type Result<T, E = CodeReaderError> = result::Result<T, E>;

#[derive(Debug)]
pub struct Code<'a> {
    data: &'a [u8]
}

impl<'a> Code<'a> {
    pub fn get_locals_reader(&self) -> Result<LocalsReader> {
        Ok(LocalsReader::new(self.data)?)
    }

    pub fn get_instruction_reader(&self, locals_iteration_proof: LocalsIterationProof) -> Result<InstructionReader> {
        let buffer = &self.data[locals_iteration_proof.position..];
        Ok(InstructionReader::new(buffer)?)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct CodeSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CodeReaderError {
    BinaryReaderError(BinaryReaderError),
}

impl From<BinaryReaderError> for CodeReaderError {
    fn from(e: BinaryReaderError) -> Self {
        CodeReaderError::BinaryReaderError(e)
    }
}

impl<'a> CodeSectionReader<'a> {
    pub(crate) fn new(buffer: &'a [u8]) -> BinaryReaderResult<CodeSectionReader<'a>> {
        let mut reader = BinaryReader::new(buffer);
        let count = reader.read_leb128_u32()?;
        Ok(CodeSectionReader { reader, count })
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read<'b>(&mut self) -> Result<Code<'b>>
        where 'a: 'b
    {
        Ok(self.read_code()?)
    }

    fn read_code<'b>(&mut self) -> Result<Code<'b>>
        where 'a: 'b
    {
        let data = self.reader.read_bytes_vec()?;
        Ok(Code { data })
    }
}

impl<'a> SectionReader for CodeSectionReader<'a> {
    type Item = Code<'a>;
    type Error = CodeReaderError;

    fn read(&mut self) -> Result<Self::Item, Self::Error> {
        self.read()
    }

    fn get_count(&self) -> u32 {
        self.get_count()
    }
}

impl<'a> IntoIterator for CodeSectionReader<'a> {
    type Item = Result<Code<'a>>;
    type IntoIter = SectionItemIterator<CodeSectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionItemIterator::new(self)
    }
}

pub struct LocalsReader<'bin> {
    reader: BinaryReader<'bin>,
    count: u32,
    remaining_items: u32,
}

pub struct LocalsIterationProof {
    position: usize,
}

impl<'a> LocalsReader<'a> {
    pub(crate) fn new(buffer: &'a [u8]) -> BinaryReaderResult<LocalsReader<'a>> {
        let mut reader = BinaryReader::new(buffer);
        let count = reader.read_leb128_u32()?;
        Ok(LocalsReader { reader, count, remaining_items: count })
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read(&mut self) -> Result<Locals> {
        let count = self.reader.read_leb128_u32()?;
        let value_type = self.reader.read_value_type()?;
        self.remaining_items -= 1;
        Ok(Locals { count, value_type })
    }

    pub fn get_iteration_proof(&mut self) -> Result<LocalsIterationProof> {
        for _ in 0..self.remaining_items {
            let _ = self.read()?;
        }
        Ok(LocalsIterationProof { position: self.reader.get_position()})
    }
}

pub struct LocalsIterator<'bin, 'loc> {
    reader: &'loc mut LocalsReader<'bin>,
    error: bool,
}

impl<'bin, 'loc> LocalsIterator<'bin, 'loc> {
    fn new(reader: &'loc mut LocalsReader<'bin>) -> LocalsIterator<'bin, 'loc> {
        LocalsIterator { reader, error: false }
    }
}

impl<'bin, 'loc> Iterator for LocalsIterator<'bin, 'loc> {
    type Item = Result<Locals>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.remaining_items == 0 || self.error {
            None
            //TODO:Ensure that no bytes are left over
            // } else if self.error {
            //     None
        } else {
            let result = self.reader.read();
            self.error = result.is_err();
            // self.reader.remaining_items -= 1;
            Some(result)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.reader.get_count() as usize;
        (count, Some(count))
    }
}

impl<'bin, 'loc> IntoIterator for &'loc mut LocalsReader<'bin>
    where 'bin: 'loc
{
    type Item = Result<Locals>;
    type IntoIter = LocalsIterator<'bin, 'loc>;

    fn into_iter(self) -> LocalsIterator<'bin, 'loc>
    {
        LocalsIterator::new(self)
    }
}