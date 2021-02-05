const WASM_MAGIC_NUMBER: &[u8; 4] = b"\0asm";
const WASM_SUPPORTED_VERSION: u32 = 0x1;

#[derive(PartialEq, Eq, Debug)]
pub enum PreambleValidationError {
    BadVersion,
    BadMagicNumber,
}

pub(crate) fn validate_preamble(magic_number: &[u8; 4], version: u32) -> Result<(), PreambleValidationError> {
    if magic_number == WASM_MAGIC_NUMBER {
        if version == WASM_SUPPORTED_VERSION {
            Ok(())
        } else {
            Err(PreambleValidationError::BadVersion)
        }
    } else {
        Err(PreambleValidationError::BadMagicNumber)
    }
}