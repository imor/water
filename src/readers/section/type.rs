use crate::readers::binary::{BinaryReader, BinaryReaderError};
use crate::types::{FunctionType, ValueType};
use crate::readers::binary::Result as BinaryReaderResult;
use std::result;
use crate::readers::common::{SectionReader, SectionItemIterator};

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TypeSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

#[derive(PartialEq, Eq, Debug)]
pub enum TypeReaderError {
    BinaryReaderError(BinaryReaderError),
    InvalidLeadingByte,
}

impl From<BinaryReaderError> for TypeReaderError {
    fn from(e: BinaryReaderError) -> Self {
        TypeReaderError::BinaryReaderError(e)
    }
}

pub type Result<T, E = TypeReaderError> = result::Result<T, E>;

impl<'a> TypeSectionReader<'a> {
    pub(crate) fn new(buffer: &'a [u8]) -> BinaryReaderResult<TypeSectionReader<'a>> {
        let mut reader = BinaryReader::new(buffer);
        let count = reader.read_leb128_u32()?;
        Ok(TypeSectionReader { reader, count })
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read(&mut self) -> Result<FunctionType> {
        let byte = self.reader.read_byte()?;
        match byte {
            0x60 => self.read_func_type(),
            _ => Err(TypeReaderError::InvalidLeadingByte),
        }
    }

    fn read_func_type(&mut self) -> Result<FunctionType> {
        let params = self.read_types_vec()?;
        let results = self.read_types_vec()?;
        Ok(FunctionType { params, results })
    }

    fn read_types_vec(&mut self) -> Result<Box<[ValueType]>> {
        let len = self.reader.read_leb128_u32()?;
        let mut types = Vec::with_capacity(len as usize);
        for _ in 0..len {
            types.push(self.reader.read_value_type()?);
        }
        Ok(types.into_boxed_slice())
    }
}

impl<'a> SectionReader for TypeSectionReader<'a> {
    type Item = FunctionType;
    type Error = TypeReaderError;

    fn read(&mut self) -> Result<Self::Item, Self::Error> {
        self.read()
    }

    fn get_count(&self) -> u32 {
        self.get_count()
    }
}

impl<'a> IntoIterator for TypeSectionReader<'a> {
    type Item = Result<FunctionType>;
    type IntoIter = SectionItemIterator<TypeSectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionItemIterator::new(self)
    }
}