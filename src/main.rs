use std::io;
use std::fs::File;
use std::io::{BufReader, Error, Read};
use water::{ParseError, Parser, Chunk, SectionReader, TypeReaderError, ImportReaderError, FunctionReaderError, ExportReaderError, TableReaderError, MemoryReaderError, GlobalReaderError, StartReaderError, ElementReaderError, DataReaderError, InstructionReaderError, CodeReaderError, Validator, ValidationError};

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
    let mut validator = Validator::new();

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
                    SectionReader::Type(reader) => {
                        println!("Found type section.");
                        for tipe in reader {
                            println!("Found func type {:?}", tipe);
                        }
                    },
                    SectionReader::Import(reader) => {
                        println!("Found import section.");
                        for import in reader {
                            println!("Found import {:?}", import);
                        }
                    },
                    SectionReader::Function(reader) => {
                        println!("Found function section.");
                        for type_index in reader {
                            println!("Found type index {:?}", type_index);
                        }
                    },
                    SectionReader::Table(reader) => {
                        println!("Found table section.");
                        for table in reader {
                            println!("Found table {:?}", table);
                        }
                    },
                    SectionReader::Memory(reader) => {
                        println!("Found memory section.");
                        for memory in reader {
                            println!("Found memory {:?}", memory);
                        }
                    },
                    SectionReader::Global(reader) => {
                        println!("Found global section.");
                        for global in reader {
                            let global = global?;
                            println!("Found global {:?}", global);
                            for instruction in global.instruction_reader {
                                let instruction = instruction?;
                                println!("Instruction: {:?}", instruction);
                            }
                        }
                    },
                    SectionReader::Export(reader) => {
                        println!("Found export section.");
                        for export in reader {
                            println!("Found export {:?}", export);
                        }
                    },
                    SectionReader::Start(reader) => {
                        println!("Found start section with func index {:?}.", reader.get_func_index())
                    },
                    SectionReader::Element(reader) => {
                        println!("Found element section.");
                        for element_segment in reader {
                            let element_segment = element_segment?;
                            println!("Found element segment {:?}", element_segment);
                            for instruction in element_segment.instruction_reader {
                                let instruction = instruction?;
                                println!("Instruction: {:?}", instruction);
                            }
                        }
                    },
                    SectionReader::Code(reader) => {
                        println!("Found code section.");
                        for code in reader {
                            let code = code?;
                            println!("Found code entry {:?}", code);
                            let mut locals_reader = code.get_locals_reader()?;
                            for locals in &mut locals_reader {
                                let locals = locals?;
                                println!("Locals: {:?}", locals);
                            }
                            let locals_iteration_proof = locals_reader.get_iteration_proof()?;
                            let instruction_reader = code.get_instruction_reader(locals_iteration_proof)?;
                            for instruction in instruction_reader {
                                let instruction = instruction?;
                                println!("Instruction: {:?}", instruction);
                            }
                        }
                    },
                    SectionReader::Data(reader) => {
                        println!("Found data section.");
                        for data_segment in reader {
                            let data_segment = data_segment?;
                            println!("Found data segment {:?}", data_segment);
                            for instruction in data_segment.instruction_reader {
                                let instruction = instruction?;
                                println!("Instruction: {:?}", instruction);
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
