use crate::binary_reader::{BinaryReader, BinaryReaderError};
use crate::ParseError::{InnerError, UnneededBytes};
use crate::{CustomSectionReader, CodeSectionReader};
use crate::TypeSectionReader;
use crate::ImportSectionReader;
use crate::FunctionSectionReader;
use crate::TableSectionReader;
use crate::MemorySectionReader;
use crate::GlobalSectionReader;
use crate::ExportSectionReader;
use crate::StartSectionReader;
use crate::ElementSectionReader;
use crate::DataSectionReader;

#[derive(PartialEq, Eq, Debug)]
pub enum SectionReader<'a> {
    Custom(CustomSectionReader<'a>),
    Type(TypeSectionReader<'a>),
    Import(ImportSectionReader<'a>),
    Function(FunctionSectionReader<'a>),
    Table(TableSectionReader<'a>),
    Memory(MemorySectionReader<'a>),
    Global(GlobalSectionReader<'a>),
    Export(ExportSectionReader<'a>),
    Start(StartSectionReader<'a>),
    Element(ElementSectionReader<'a>),
    Code(CodeSectionReader<'a>),
    Data(DataSectionReader<'a>),
    Unknown(u8),
}

#[derive(PartialEq, Eq, Debug)]
pub enum Chunk<'a> {
    Header(u32),
    Section(SectionReader<'a>),
    Done,
}

#[derive(PartialEq, Eq, Debug)]
pub enum ParseError {
    UnneededBytes,
    InnerError(BinaryReaderError),
}

impl From<BinaryReaderError> for ParseError {
    fn from(e: BinaryReaderError) -> Self {
        InnerError(e)
    }
}

enum ParserLocation {
    ModuleHeader,
    Section,
    End,
}

pub struct Parser {
    location: ParserLocation,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            location: ParserLocation::ModuleHeader,
        }
    }

    pub fn parse<'a>(&mut self, buffer: &'a [u8]) -> Result<(usize, Chunk<'a>), ParseError> {
        let mut reader = BinaryReader::new(buffer);
        match self.location {
            ParserLocation::ModuleHeader => {
                let (consumed, version) = reader.read_file_header()?;
                self.location = ParserLocation::Section;
                Ok((consumed, Chunk::Header(version)))
            },
            ParserLocation::Section => {
                if buffer.is_empty() {
                    self.location = ParserLocation::End;
                    Ok((0, Chunk::Done))
                } else {
                    let id = reader.read_byte()?;
                    let size = reader.read_u32()? as usize;
                    Ok((reader.position + size,
                        Chunk::Section(Self::create_section_reader(&buffer[reader.position..reader.position + size], id)?)))
                }
            }
            ParserLocation::End => {
                if !buffer.is_empty() {
                    Err(UnneededBytes)
                } else {
                    Ok((0, Chunk::Done))
                }
            }
        }
    }

    fn create_section_reader(buffer: &[u8], id: u8) -> Result<SectionReader, ParseError> {
        Ok(match id {
            0 => SectionReader::Custom(CustomSectionReader::new(buffer)?),
            1 => SectionReader::Type(TypeSectionReader::new(buffer)?),
            2 => SectionReader::Import(ImportSectionReader::new(buffer)?),
            3 => SectionReader::Function(FunctionSectionReader::new(buffer)?),
            4 => SectionReader::Table(TableSectionReader::new(buffer)?),
            5 => SectionReader::Memory(MemorySectionReader::new(buffer)?),
            6 => SectionReader::Global(GlobalSectionReader::new(buffer)?),
            7 => SectionReader::Export(ExportSectionReader::new(buffer)?),
            8 => SectionReader::Start(StartSectionReader::new(buffer)?),
            9 => SectionReader::Element(ElementSectionReader::new(buffer)?),
            10 => SectionReader::Code(CodeSectionReader::new(buffer)?),
            11 => SectionReader::Data(DataSectionReader::new(buffer)?),
            id => SectionReader::Unknown(id),
        })
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::Parser;
    use crate::binary_reader::BinaryReaderError::{UnexpectedEof, BadVersion};
    use crate::Chunk::Header;
    use crate::ParseError::InnerError;

    #[test]
    fn parse_header_from_empty() {
        let mut parser = Parser::new();
        let result = parser.parse(&[]);
        assert_eq!(Err(InnerError(UnexpectedEof)), result);
    }

    #[test]
    fn parse_header_bad_magic_no() {
        let mut parser = Parser::new();
        let result = parser.parse(b"\0as");
        assert_eq!(Err(InnerError(UnexpectedEof)), result);
    }

    #[test]
    fn parse_header_only_magic_no() {
        let mut parser = Parser::new();
        let result = parser.parse(b"\0asm");
        assert_eq!(Err(InnerError(UnexpectedEof)), result);
    }

    #[test]
    fn parse_header_bad_version() {
        let mut parser = Parser::new();
        let result = parser.parse(b"\0asm\x02\0\0\0");
        assert_eq!(Err(InnerError(BadVersion)), result);
    }

    #[test]
    fn parse_good_header() {
        let mut parser = Parser::new();
        let result = parser.parse(b"\0asm\x01\0\0\0");
        assert_eq!(Ok((8, Header(1))), result);
    }

    //#[test]
    // fn unneeded_bytes_test() {
    //     let mut parser = Parser::new();
    //     let _ = parser.parse(b"\0asm\x01\0\0\0");
    //     let result = parser.parse(b"MoreBytes");
    //     assert_eq!(Err(UnneededBytes), result);
    // }
}