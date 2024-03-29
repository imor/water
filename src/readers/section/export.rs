use crate::readers::binary::{BinaryReader, BinaryReaderError};
use crate::readers::binary::Result as BinaryReaderResult;
use std::result;
use crate::types::{Export, ExportDescriptor, FuncIndex, TableIndex, MemoryIndex, GlobalIndex};
use crate::readers::common::{SectionReader, SectionItemIterator};

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ExportSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

#[derive(Eq, PartialEq, Debug)]
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
        let count = reader.read_leb128_u32()?;
        Ok(ExportSectionReader { reader, count })
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read<'b>(&mut self) -> Result<Export<'b>>
        where 'a: 'b
    {
        let name = self.reader.read_string()?;
        let export_desc = self.read_export_desc()?;
        Ok(Export { name, export_descriptor: export_desc })
    }

    fn read_export_desc(&mut self) -> Result<ExportDescriptor> {
        match self.reader.read_byte()? {
            0x00 => {
                let func_index = FuncIndex(self.reader.read_leb128_u32()?);
                Ok(ExportDescriptor::Func { func_index })
            },
            0x01 => {
                let table_index = TableIndex(self.reader.read_leb128_u32()?);
                Ok(ExportDescriptor::Table { table_index })
            },
            0x02 => {
                let memory_index = MemoryIndex(self.reader.read_leb128_u32()?);
                Ok(ExportDescriptor::Memory { memory_index })
            },
            0x03 => {
                let global_index = GlobalIndex(self.reader.read_leb128_u32()?);
                Ok(ExportDescriptor::Global { global_index })
            },
            _ => Err(ExportReaderError::InvalidExportDescByte)
        }
    }
}

impl<'a> SectionReader for ExportSectionReader<'a> {
    type Item = Export<'a>;
    type Error = ExportReaderError;

    fn read(&mut self) -> Result<Self::Item, Self::Error> {
        self.read()
    }

    fn get_count(&self) -> u32 {
        self.get_count()
    }
}

impl<'a> IntoIterator for ExportSectionReader<'a> {
    type Item = Result<Export<'a>>;
    type IntoIter = SectionItemIterator<ExportSectionReader<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionItemIterator::new(self)
    }
}
