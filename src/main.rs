use std::io;
use std::fs::File;
use std::io::{BufReader, Error, Read};
use water::{ParseError, Parser, Chunk, SectionReader};

#[derive(Debug)]
enum MyError {
    IoError(io::Error),
    ParseError(ParseError),
}

impl From<io::Error> for MyError {
    fn from(e: Error) -> Self {
        MyError::IoError(e)
    }
}

impl From<ParseError> for MyError {
    fn from(e: ParseError) -> Self {
        MyError::ParseError(e)
    }
}

fn main() -> Result<(), MyError> {
    let f = File::open("hello.wasm")?;
    let mut reader = BufReader::new(f);
    let mut v = Vec::new();
    reader.read_to_end(&mut v)?;

    let mut parser = Parser::new();

    loop {
        let consumed = match parser.parse(&v)? {
            (consumed, Chunk::Header(version)) => {
                println!("Found header with version {}", version);
                consumed
            },
            (consumed, Chunk::Section(section)) => {
                match section {
                    SectionReader::Custom => println!("Found custom section."),
                    SectionReader::Type(_reader) => println!("Found type section."),
                    SectionReader::Import => println!("Found import section."),
                    SectionReader::Function => println!("Found function section."),
                    SectionReader::Table => println!("Found table section."),
                    SectionReader::Memory => println!("Found memory section."),
                    SectionReader::Global => println!("Found global section."),
                    SectionReader::Export => println!("Found export section."),
                    SectionReader::Start => println!("Found start section."),
                    SectionReader::Element => println!("Found element section."),
                    SectionReader::Code => println!("Found code section."),
                    SectionReader::Data => println!("Found data section."),
                    SectionReader::Unknown => println!("Found unknown section."),
                }
                consumed
            }
            (_, Chunk::Done) => {
                break;
            },
        };
        println!("Consumed {} bytes", consumed);
        v.drain(..consumed);
    }
    Ok(())
}
