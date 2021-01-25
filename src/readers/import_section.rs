use crate::binary_reader::{BinaryReader, BinaryReaderError};
use crate::binary_reader::Result as BinaryReaderResult;
use std::result;
use crate::types::{Import, ImportDesc, TypeIndex};

#[derive(Eq, PartialEq, Debug)]
pub struct ImportSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

#[derive(Debug)]
pub enum ImportReaderError {
    BinaryReaderError(BinaryReaderError),
    InvalidImportDescByte,
}

impl From<BinaryReaderError> for ImportReaderError {
    fn from(e: BinaryReaderError) -> Self {
        ImportReaderError::BinaryReaderError(e)
    }
}

pub type Result<T, E = ImportReaderError> = result::Result<T, E>;

impl<'a> ImportSectionReader<'a> {
    pub(crate) fn new(buffer: &'a [u8]) -> BinaryReaderResult<ImportSectionReader<'a>> {
        let mut reader = BinaryReader::new(buffer);
        let count = reader.read_u32()?;
        Ok(ImportSectionReader { reader, count })
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read(&mut self) -> Result<Import> {
        let module_name = self.reader.read_string()?;
        let name = self.reader.read_string()?;
        let import_desc = self.read_import_desc()?;
        Ok(Import { module_name, name, import_desc })
    }

    fn read_import_desc(&mut self) -> Result<ImportDesc> {
        match self.reader.read_byte()? {
            0x00 => {
                let type_index = TypeIndex(self.reader.read_u32()?);
                Ok(ImportDesc::Func{ type_index })
            },
            0x01 => {
                let table_type = self.reader.read_table_type()?;
                Ok(ImportDesc::Table(table_type))
            },
            0x02 => {
                let memory_type = self.reader.read_memory_type()?;
                Ok(ImportDesc::Memory(memory_type))
            },
            0x03 => {
                let global_type = self.reader.read_global_type()?;
                Ok(ImportDesc::Global(global_type))
            },
            _ => Err(ImportReaderError::InvalidImportDescByte)
        }
    }
}