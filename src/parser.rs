use crate::binary_reader::{BinaryReader, BinaryReaderError};

pub enum Section {
    Header(u32),
    Done,
}

#[derive(Debug)]
pub struct ParseError {
    message: String,
}

impl From<BinaryReaderError> for ParseError {
    fn from(e: BinaryReaderError) -> Self {
        ParseError { message: e.message }
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