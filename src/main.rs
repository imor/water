use std::io;
use std::fs::File;
use std::io::{BufReader, Error, Read};
use water::{ParseError, Parser, Chunk, Section};

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
                    Section::Custom => println!("Found custom section."),
                    Section::Type => println!("Found type section."),
                    Section::Import => println!("Found import section."),
                    Section::Function => println!("Found function section."),
                    Section::Table => println!("Found table section."),
                    Section::Memory => println!("Found memory section."),
                    Section::Global => println!("Found global section."),
                    Section::Export => println!("Found export section."),
                    Section::Start => println!("Found start section."),
                    Section::Element => println!("Found element section."),
                    Section::Code => println!("Found code section."),
                    Section::Data => println!("Found data section."),
                    Section::Unknown => println!("Found unknown section."),
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
