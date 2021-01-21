use crate::binary_reader::{BinaryReader, BinaryReaderError};
use crate::binary_reader::Result as BinaryReaderResult;
use std::result;
use crate::types::{Export, ExportDesc, FuncIndex};

#[derive(Eq, PartialEq, Debug)]
pub struct ExportSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

#[derive(Debug)]
pub enum ExportReaderError {
    BinaryReaderError(BinaryReaderError),
    InvalidExportDescByte,
}

impl From<BinaryReaderError> for ExportReaderError {
    fn from(e: BinaryReaderError) -> Self {
        ExportReaderError::BinaryReaderError(e)
    }
}

pub type Result<T, E = ExportReaderError> = result::Result<T, E>;

impl<'a> ExportSectionReader<'a> {
    pub(crate) fn new(buffer: &'a [u8]) -> BinaryReaderResult<ExportSectionReader<'a>> {
        let mut reader = BinaryReader::new(buffer);
        let count = reader.read_var_u32()?;
        Ok(ExportSectionReader { reader, count })
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read(&mut self) -> Result<Export> {
        let name = self.reader.read_string()?;
        let export_desc = self.read_export_desc()?;
        Ok(Export { name, export_desc })
    }

    fn read_export_desc(&mut self) -> Result<ExportDesc> {
        match self.reader.read_u8()? {
            0x00 => {
                let func_index = FuncIndex(self.reader.read_var_u32()?);
                Ok(ExportDesc::Func { func_index })
            },
            0x01 => {
                let table_index = self.reader.read_var_u32()?;
                Ok(ExportDesc::Table { table_index })
            },
            0x02 => {
                let memory_index = self.reader.read_var_u32()?;
                Ok(ExportDesc::Memory { memory_index })
            },
            0x03 => {
                let global_index = self.reader.read_var_u32()?;
                Ok(ExportDesc::Global { global_index })
            },
            _ => Err(ExportReaderError::InvalidExportDescByte)
        }
    }
}
