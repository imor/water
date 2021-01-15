use std::io;
use std::fs::File;
use std::io::{BufReader, Error, Read};
use water::{ParseError, Parser, Section};

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
        match parser.parse(&v)? {
            (consumed, Section::Header(version)) => {
                println!("Found header with version {}", version);
                v.drain(..consumed);
            },
            (_, Section::Done) => {
                println!("Finished parsing");
                break
            },
        }
    }
    Ok(())
}
