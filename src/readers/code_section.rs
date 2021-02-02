use crate::readers::binary::{BinaryReader, BinaryReaderError};
use crate::readers::binary::Result as BinaryReaderResult;
use std::result;
use crate::types::Locals;
use crate::InstructionReader;

pub type Result<T, E = CodeReaderError> = result::Result<T, E>;

#[derive(Debug)]
pub struct Code<'a> {
    data: &'a [u8]
}

impl<'a> Code<'a> {
    pub fn get_locals_reader(&self) -> Result<LocalsReader> {
        Ok(LocalsReader::new(self.data)?)
    }

    pub fn get_instruction_reader(&self, locals_reader: LocalsReader) -> Result<InstructionReader> {
        let buffer = &self.data[locals_reader.reader.position..];
        Ok(InstructionReader::new(buffer)?)
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct CodeSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

#[derive(Debug)]
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
        let count = reader.read_u32()?;
        Ok(CodeSectionReader { reader, count })
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read(&mut self) -> Result<Code> {
        Ok(self.read_code()?)
    }

    fn read_code(&mut self) -> Result<Code> {
        let size_bytes = self.reader.read_u32()? as usize;
        println!("code size: {}", size_bytes);
        //TODO: check at all places where a slice is created that it
        //is within the bounds of the buffer
        let data = &self.reader.buffer[self.reader.position..self.reader.position + size_bytes];
        self.reader.position += size_bytes;
        Ok(Code { data })
    }
}

pub struct LocalsReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> LocalsReader<'a> {
    pub(crate) fn new(buffer: &'a [u8]) -> BinaryReaderResult<LocalsReader<'a>> {
        let mut reader = BinaryReader::new(buffer);
        let count = reader.read_u32()?;
        Ok(LocalsReader { reader, count })
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read(&mut self) -> Result<Locals> {
        let count = self.reader.read_u32()?;
        let value_type = self.reader.read_value_type()?;
        Ok(Locals { count, value_type })
    }
}
