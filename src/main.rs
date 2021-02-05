use std::io;
use std::fs::File;
use std::io::{BufReader, Error, Read};
use water::{ParseError, Parser, Chunk, SectionReader, TypeReaderError, ImportReaderError, FunctionReaderError, ExportReaderError, TableReaderError, MemoryReaderError, GlobalReaderError, StartReaderError, ElementReaderError, DataReaderError, InstructionReaderError, Instruction, CodeReaderError, Validator, ValidationError};

#[derive(Debug)]
enum MyError {
    Io(io::Error),
    Parse(ParseError),
    TypeReader(TypeReaderError),
    ImportReader(ImportReaderError),
    FunctionReader(FunctionReaderError),
    ExportReader(ExportReaderError),
    TableReader(TableReaderError),
    MemoryReader(MemoryReaderError),
    GlobalReader(GlobalReaderError),
    StartReader(StartReaderError),
    ElementReader(ElementReaderError),
    CodeReader(CodeReaderError),
    DataReader(DataReaderError),
    InstructionReader(InstructionReaderError),
    Validation(ValidationError),
}

impl From<io::Error> for MyError {
    fn from(e: Error) -> Self {
        MyError::Io(e)
    }
}

impl From<ParseError> for MyError {
    fn from(e: ParseError) -> Self {
        MyError::Parse(e)
    }
}

impl From<TypeReaderError> for MyError {
    fn from(e: TypeReaderError) -> Self {
        MyError::TypeReader(e)
    }
}

impl From<ImportReaderError> for MyError {
    fn from(e: ImportReaderError) -> Self {
        MyError::ImportReader(e)
    }
}

impl From<FunctionReaderError> for MyError {
    fn from(e: FunctionReaderError) -> Self {
        MyError::FunctionReader(e)
    }
}

impl From<ExportReaderError> for MyError {
    fn from(e: ExportReaderError) -> Self {
        MyError::ExportReader(e)
    }
}

impl From<TableReaderError> for MyError {
    fn from(e: TableReaderError) -> Self {
        MyError::TableReader(e)
    }
}

impl From<MemoryReaderError> for MyError {
    fn from(e: MemoryReaderError) -> Self {
        MyError::MemoryReader(e)
    }
}

impl From<GlobalReaderError> for MyError {
    fn from(e: GlobalReaderError) -> Self {
        MyError::GlobalReader(e)
    }
}

impl From<StartReaderError> for MyError {
    fn from(e: StartReaderError) -> Self {
        MyError::StartReader(e)
    }
}

impl From<ElementReaderError> for MyError {
    fn from(e: ElementReaderError) -> Self {
        MyError::ElementReader(e)
    }
}

impl From<CodeReaderError> for MyError {
    fn from(e: CodeReaderError) -> Self {
        MyError::CodeReader(e)
    }
}

impl From<DataReaderError> for MyError {
    fn from(e: DataReaderError) -> Self {
        MyError::DataReader(e)
    }
}

impl From<InstructionReaderError> for MyError {
    fn from(e: InstructionReaderError) -> Self {
        MyError::InstructionReader(e)
    }
}

impl From<ValidationError> for MyError {
    fn from(e: ValidationError) -> Self {
        MyError::Validation(e)
    }
}

fn main() -> Result<(), MyError> {
    // let f = File::open("hello.wasm")?;
    let f = File::open("C:/Users/raminder.singh/Downloads/main_bg.wasm")?;
    let mut reader = BufReader::new(f);
    let mut v = Vec::new();
    reader.read_to_end(&mut v)?;

    let mut parser = Parser::new();
    let validator = Validator::new();

    loop {
        let parse_result = parser.parse(&v)?;
        let _ = validator.validate(&parse_result.1)?;
        let consumed = match parse_result {
            (consumed, Chunk::Preamble(magic_number, version)) => {
                println!("Found header with magic_number: {:?} and version {}", magic_number, version);
                consumed
            },
            (consumed, Chunk::Section(section)) => {
                match section {
                    SectionReader::Custom(reader) => {
                        println!("Found custom section with name {} and {} bytes data.", reader.get_name(), reader.get_data().len());
                    },
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
                    SectionReader::Function(mut reader) => {
                        println!("Found function section.");
                        let count = reader.get_count();
                        for _ in 0..count {
                            let type_index = reader.read()?;
                            println!("Found type index {:?}", type_index);
                        }
                    },
                    SectionReader::Table(mut reader) => {
                        println!("Found table section.");
                        let count = reader.get_count();
                        for _ in 0..count {
                            let table = reader.read()?;
                            println!("Found table {:?}", table);
                        }
                    },
                    SectionReader::Memory(mut reader) => {
                        println!("Found memory section.");
                        let count = reader.get_count();
                        for _ in 0..count {
                            let memory = reader.read()?;
                            println!("Found memory {:?}", memory);
                        }
                    },
                    SectionReader::Global(mut reader) => {
                        println!("Found global section.");
                        let count = reader.get_count();
                        for _ in 0..count {
                            let global = reader.read()?;
                            println!("Found global {:?}", global);
                        }
                    },
                    SectionReader::Export(mut reader) => {
                        println!("Found export section.");
                        let count = reader.get_count();
                        for _ in 0..count {
                            let export = reader.read()?;
                            println!("Found export {:?}", export);
                        }
                    },
                    SectionReader::Start(reader) => {
                        println!("Found start section with func index {:?}.", reader.get_func_index())
                    },
                    SectionReader::Element(mut reader) => {
                        let count = reader.get_count();
                        println!("Found element section with {} elements.", count);
                        for _ in 0..count {
                            let mut element_type = reader.read()?;
                            println!("Found element type {:?}", element_type);
                            loop {
                                let instruction = element_type.instruction_reader.read()?;
                                println!("Instruction: {:?}", instruction);
                                if let Instruction::End = instruction {
                                    break;
                                }
                            }
                        }
                    },
                    SectionReader::Code(mut reader) => {
                        let count = reader.get_count();
                        println!("Found code section with {} code entries.", count);
                        for _ in 0..count {
                            let code = reader.read()?;
                            println!("Found code entry {:?}", code);
                            let mut locals_reader = code.get_locals_reader()?;
                            let locals_count = locals_reader.get_count();
                            println!("Found {} locals", locals_count);
                            for _ in 0..locals_count {
                                let locals = locals_reader.read()?;
                                println!("Locals: {:?}", locals);
                            }
                            let mut instruction_reader = code.get_instruction_reader(locals_reader)?;
                            loop {
                                if instruction_reader.eof() {
                                    break;
                                }
                                let instruction = instruction_reader.read();
                                if let Ok(instruction) = instruction {
                                    println!("Instruction: {:?}", instruction);
                                } else {
                                    println!("Error while reading instruction: {:?}", instruction)
                                }
                            }
                        }
                    },
                    SectionReader::Data(mut reader) => {
                        let count = reader.get_count();
                        println!("Found data section with {} data elements.", count);
                        for _ in 0..count {
                            let mut data_type = reader.read()?;
                            println!("Found data type {:?}", data_type);
                            loop {
                                let instruction = data_type.instruction_reader.read()?;
                                println!("Instruction: {:?}", instruction);
                                if let Instruction::End = instruction {
                                    break;
                                }
                            }
                        }
                    },
                    SectionReader::Unknown(id) => println!("Found unknown section with id {}.", id),
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
