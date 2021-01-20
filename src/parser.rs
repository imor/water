use crate::binary_reader::{BinaryReader, BinaryReaderError};
use crate::ParseError::{InnerError, UnneededBytes};
use crate::readers::{TypeSectionReader, ImportSectionReader, FunctionSectionReader};

#[derive(PartialEq, Eq, Debug)]
pub enum SectionReader<'a> {
    Custom,
    Type(TypeSectionReader<'a>),
    Import(ImportSectionReader<'a>),
    Function(FunctionSectionReader<'a>),
    Table,
    Memory,
    Global,
    Export,
    Start,
    Element,
    Code,
    Data,
    Unknown(u8),
}

// impl From<u8> for SectionReader {
//     fn from(val: u8) -> Self {
//         match val {
//             0 => SectionReader::Custom,
//             1 => SectionReader::Type,
//             2 => SectionReader::Import,
//             3 => SectionReader::Function,
//             4 => SectionReader::Table,
//             5 => SectionReader::Memory,
//             6 => SectionReader::Global,
//             7 => SectionReader::Export,
//             8 => SectionReader::Start,
//             9 => SectionReader::Element,
//             10 => SectionReader::Code,
//             11 => SectionReader::Data,
//             _ => SectionReader::Unknown,
//         }
//     }
// }

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
                if buffer.len() == 0 {
                    self.location = ParserLocation::End;
                    Ok((0, Chunk::Done))
                } else {
                    let id = reader.read_u8()?;
                    let size = reader.read_var_u32()?;
                    Ok((reader.position + size as usize, Chunk::Section(Self::create_section_reader(&buffer[reader.position..], id)?)))
                }
            }
            ParserLocation::End => {
                if buffer.len() > 0 {
                    Err(UnneededBytes)
                } else {
                    Ok((0, Chunk::Done))
                }
            }
        }
    }

    fn create_section_reader(buffer: &[u8], id: u8) -> Result<SectionReader, ParseError> {
        Ok(match id {
            1 => SectionReader::Type(TypeSectionReader::new(buffer)?),
            2 => SectionReader::Import(ImportSectionReader::new(buffer)?),
            3 => SectionReader::Function(FunctionSectionReader::new(buffer)?),
            id => SectionReader::Unknown(id),
        })
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