#[derive(Eq, PartialEq, Debug)]
pub enum Type {
    I32,
    I64,
    F32,
    F64,
}

#[derive(Eq, PartialEq, Debug)]
pub struct FuncType {
    pub(crate) params: Box<[Type]>,
    pub(crate) results: Box<[Type]>,
}

#[derive(Debug)]
pub enum ImportDesc {
    Func { type_index: u32},
    Table,
    Memory,
    Global,
}

#[derive(Debug)]
pub struct Import<'a> {
    pub(crate) module_name: &'a str,
    pub(crate) name: &'a str,
    pub(crate) import_desc: ImportDesc
}