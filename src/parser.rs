use crate::binary_reader::{BinaryReader, BinaryReaderError};
use crate::ParseError::{InnerError, UnneededBytes};

#[derive(PartialEq, Eq, Debug)]
pub enum Section {
    Custom,
    Type,
    Import,
    Function,
    Table,
    Memory,
    Global,
    Export,
    Start,
    Element,
    Code,
    Data,
    Unknown,
}

impl From<u8> for Section {
    fn from(val: u8) -> Self {
        match val {
            0 => Section::Custom,
            1 => Section::Type,
            2 => Section::Import,
            3 => Section::Function,
            4 => Section::Table,
            5 => Section::Memory,
            6 => Section::Global,
            7 => Section::Export,
            8 => Section::Start,
            9 => Section::Element,
            10 => Section::Code,
            11 => Section::Data,
            _ => Section::Unknown,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Chunk {
    Header(u32),
    Section(Section),
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

    pub fn parse(&mut self, buffer: &[u8]) -> Result<(usize, Chunk), ParseError> {
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
                    let (_, id) = reader.read_u8()?;
                    let (consumed, size) = reader.read_var_u32()?;
                    Ok((consumed + size as usize, Chunk::Section(id.into())))
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