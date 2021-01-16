use crate::binary_reader::{BinaryReader, BinaryReaderError};
use crate::ParseError::{InnerError, UnneededBytes};

#[derive(PartialEq, Eq, Debug)]
pub enum Section {
    Header(u32),
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
    // Section,
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

    pub fn parse(&mut self, buffer: &[u8]) -> Result<(usize, Section), ParseError> {
        let mut reader = BinaryReader::new(buffer);
        match self.location {
            ParserLocation::ModuleHeader => {
                let (consumed, version) = reader.read_file_header()?;
                self.location = ParserLocation::End;
                // self.location = ParserLocation::Section;
                Ok((consumed, Section::Header(version)))
            },
            // ParserLocation::Section => {
            //     if buffer.len() == 0 {
            //         self.location = ParserLocation::End;
            //         Ok((0, Section::Done))
            //     } else {
            //         let (consumed, id) = reader.read_u8()?;
            //         //TODO:Fix
            //         Ok((0, Section::Done))
            //     }
            // }
            ParserLocation::End => {
                if buffer.len() > 0 {
                    Err(UnneededBytes)
                } else {
                    Ok((0, Section::Done))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Parser;
    use crate::binary_reader::BinaryReaderError::{UnexpectedEof, BadVersion};
    use crate::Section::Header;
    use crate::ParseError::{InnerError, UnneededBytes};

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

    #[test]
    fn unneeded_bytes_test() {
        let mut parser = Parser::new();
        let _ = parser.parse(b"\0asm\x01\0\0\0");
        let result = parser.parse(b"MoreBytes");
        assert_eq!(Err(UnneededBytes), result);
    }
}