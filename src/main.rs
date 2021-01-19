use std::io;
use std::fs::File;
use std::io::{BufReader, Error, Read};
use water::{ParseError, Parser, Chunk, SectionReader, TypeReaderError, ImportReaderError};

#[derive(Debug)]
enum MyError {
    IoError(io::Error),
    ParseError(ParseError),
    TypeReaderError(TypeReaderError),
    ImportReaderError(ImportReaderError),
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

impl From<TypeReaderError> for MyError {
    fn from(e: TypeReaderError) -> Self {
        MyError::TypeReaderError(e)
    }
}

impl From<ImportReaderError> for MyError {
    fn from(e: ImportReaderError) -> Self {
        MyError::ImportReaderError(e)
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
                    SectionReader::Type(mut reader) => {
                        println!("Found type section.");
                        let count = reader.get_count();
                        println!("Found {} types", count);
                        for _ in 0..count {
                            let func_type = reader.read()?;
                            println!("Found func type {:?}", func_type);
                        }
                    },
                    SectionReader::Import(mut reader) => {
                        println!("Found import section.");
                        let count = reader.get_count();
                        for _ in 0..count {
                            let import = reader.read()?;
                            println!("Found import {:?}", import);
                        }
                    },
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
