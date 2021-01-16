use crate::binary_reader::{BinaryReader, BinaryReaderError};

#[derive(PartialEq, Eq, Debug)]
pub enum Section {
    Header(u32),
    Done,
}

#[derive(PartialEq, Eq, Debug)]
pub struct ParseError {
    inner: BinaryReaderError
}

impl From<BinaryReaderError> for ParseError {
    fn from(e: BinaryReaderError) -> Self {
        ParseError { inner: e }
    }
}

enum ParserLocation {
    Header,
    End,
}

pub struct Parser {
    location: ParserLocation,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            location: ParserLocation::Header,
        }
    }

    pub fn parse(&mut self, buffer: &[u8]) -> Result<(usize, Section), ParseError> {
        let mut reader = BinaryReader::new(buffer);
        let result = match self.location {
            ParserLocation::Header => {
                let (consumed, version) = reader.read_file_header()?;
                self.location = ParserLocation::End;
                (consumed, Section::Header(version))
            },
            ParserLocation::End => {
                (0, Section::Done)
            }
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Parser, ParseError};
    use crate::binary_reader::BinaryReaderError::{UnexpectedEof, BadVersion};
    use crate::Section::Header;

    #[test]
    fn parse_header_from_empty() {
        let mut parser = Parser::new();
        let result = parser.parse(&[]);
        assert_eq!(Err(ParseError { inner: UnexpectedEof }), result);
    }

    #[test]
    fn parse_header_bad_magic_no() {
        let mut parser = Parser::new();
        let result = parser.parse(b"\0as");
        assert_eq!(Err(ParseError { inner: UnexpectedEof }), result);
    }

    #[test]
    fn parse_header_only_magic_no() {
        let mut parser = Parser::new();
        let result = parser.parse(b"\0asm");
        assert_eq!(Err(ParseError { inner: UnexpectedEof }), result);
    }

    #[test]
    fn parse_header_bad_version() {
        let mut parser = Parser::new();
        let result = parser.parse(b"\0asm\x02\0\0\0");
        assert_eq!(Err(ParseError { inner: BadVersion }), result);
    }

    #[test]
    fn parse_good_header() {
        let mut parser = Parser::new();
        let result = parser.parse(b"\0asm\x01\0\0\0");
        assert_eq!(Ok((8, Header(1))), result);
    }
}