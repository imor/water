use crate::binary_reader::{BinaryReader, BinaryReaderError};
use crate::primitives::{FuncType, Type};
use crate::binary_reader::Result as BinaryReaderResult;
use std::result;
use crate::primitives::Type::{I32, I64, F32, F64};
use crate::readers::type_section::TypeReaderError::InvalidValueTypeByte;

#[derive(Eq, PartialEq, Debug)]
pub struct TypeSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

#[derive(Debug)]
pub enum TypeReaderError {
    BinaryReaderError(BinaryReaderError),
    InvalidLeadingByte,
    InvalidValueTypeByte,
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
        let count = reader.read_var_u32()?;
        Ok(TypeSectionReader { reader, count })
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read(&mut self) -> Result<FuncType> {
        let byte = self.reader.read_u8()?;
        match byte {
            0x60 => self.read_func_type(),
            _ => Err(TypeReaderError::InvalidLeadingByte),
        }
    }

    fn read_func_type(&mut self) -> Result<FuncType> {
        let params = self.read_types_vec()?;
        let results = self.read_types_vec()?;
        Ok(FuncType { params, results })
    }

    fn read_types_vec(&mut self) -> Result<Box<[Type]>> {
        let len = self.reader.read_var_u32()?;
        let mut types = Vec::with_capacity(len as usize);
        for _ in 0..len {
            types.push(self.read_type()?);
        }
        Ok(types.into_boxed_slice())
    }

    fn read_type(&mut self) -> Result<Type> {
        match self.reader.read_u8()? {
            0x7F => Ok(I32),
            0xFE => Ok(I64),
            0x7D => Ok(F32),
            0x7C => Ok(F64),
            _ => Err(InvalidValueTypeByte)
        }
    }
}
