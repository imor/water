use std::convert::TryInto;
use crate::binary_reader::BinaryReaderError::{UnexpectedEof, BadVersion, BadMagicNumber, InvalidVaru32, InvalidElementTypeByte, InvalidLimitsByte};
use std::{result, str};
use crate::primitives::{TableType, Limits, MemoryType};

const WASM_MAGIC_NUMBER: &[u8; 4] = b"\0asm";
const WASM_SUPPORTED_VERSION: u32 = 0x1;

pub type Result<T, E = BinaryReaderError> = result::Result<T, E>;

#[derive(PartialEq, Eq, Debug)]
pub enum BinaryReaderError {
    UnexpectedEof,
    BadVersion,
    BadMagicNumber,
    InvalidVaru32,
    InvalidUtf8,
    InvalidElementTypeByte,
    InvalidLimitsByte,
}

#[derive(Eq, PartialEq, Debug)]
pub struct BinaryReader<'a> {
    buffer: &'a [u8],
    pub(crate) position: usize,
}

impl<'a> BinaryReader<'a> {
    pub fn new(buffer: &[u8]) -> BinaryReader {
        BinaryReader {
            buffer,
            position: 0,
        }
    }

    fn ensure_has_bytes(&self, n: usize) -> Result<()> {
        if self.position + n <= self.buffer.len() {
            Ok(())
        } else {
            Err(UnexpectedEof)
        }
    }

    fn read_bytes(&mut self, n: usize) -> Result<&'a [u8]> {
        self.ensure_has_bytes(n)?;
        let start = self.position;
        self.position += n;
        Ok(&self.buffer[start..self.position])
    }

    fn read_u32(&mut self) -> Result<u32> {
        self.ensure_has_bytes(4)?;
        let word = u32::from_le_bytes(
            self.buffer[self.position..self.position + 4]
                .try_into().unwrap()
        );
        self.position += 4;
        Ok(word)
    }

    pub fn read_file_header(&mut self) -> Result<(usize, u32)> {
        let magic_number = self.read_bytes(4)?;
        if magic_number == WASM_MAGIC_NUMBER {
            let version = self.read_u32()?;
            if version == WASM_SUPPORTED_VERSION {
                Ok((self.position, version))
            } else {
                Err(BadVersion)
            }
        } else {
            Err(BadMagicNumber)
        }
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        let bytes = self.read_bytes(1)?;
        Ok(bytes[0])
    }

    pub fn read_var_u32(&mut self) -> Result<u32> {
        let mut result: u32 = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_u8()?;
            result |= (byte as u32 & 0b0111_1111) << shift;
            // The fifth byte's 4 high bits must be zero
            if shift == 28 && (byte >> (32 - shift)) != 0 {
                return Err(InvalidVaru32);
            }
            shift += 7;
            if byte & 0b1000_0000 == 0 {
                break;
            }
        }
        Ok(result)
    }

    pub fn read_string(&mut self) -> Result<&'a str> {
        let len = self.read_var_u32()? as usize;
        let bytes = self.read_bytes(len)?;
        str::from_utf8(bytes).map_err(|_| BinaryReaderError::InvalidUtf8)
    }

    pub fn read_table_type(&mut self) -> Result<TableType> {
        match self.read_u8()? {
            0x70 => {
                let limits = self.read_limits()?;
                Ok(TableType { limits })
            },
            _ => Err(InvalidElementTypeByte)
        }
    }

    pub fn read_memory_type(&mut self) -> Result<MemoryType> {
        let limits = self.read_limits()?;
        Ok(MemoryType { limits })
    }

    fn read_limits(&mut self) -> Result<Limits> {
        match self.read_u8()? {
            0x00 => {
                let min = self.read_var_u32()?;
                let max = None;
                Ok(Limits { min, max })
            },
            0x01 => {
                let min = self.read_var_u32()?;
                let max = Some(self.read_var_u32()?);
                Ok(Limits { min, max })
            },
            _ => Err(InvalidLimitsByte)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::binary_reader::{BinaryReader, BinaryReaderError};
    use crate::binary_reader::BinaryReaderError::{UnexpectedEof, InvalidVaru32};

    #[test]
    fn read_var_u32() {
        for item in
            [
                (vec![0b0000_0000], Ok(0u32)),
                (vec![0b0000_0001], Ok(1)),
                (vec![0b0000_0100], Ok(4)),
                (vec![0b0111_1111], Ok(127)),
                (vec![0b1111_1111], Err(UnexpectedEof)),
                (vec![0b1111_1111, 0b0000_0000], Ok(127)),
                (vec![0b1111_1111, 0b0000_0001], Ok(255)),
                (vec![0b1111_1111, 0b0111_1111], Ok(16_383)),
                (vec![0b1111_1111, 0b1111_1111, 0b0000_0001], Ok(32_767)),
                (vec![0b1111_1111, 0b1111_1111, 0b0111_1111], Ok(2_097_151)),
                (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_0001], Ok(4_194_303)),
                (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0111_1111], Ok(268_435_455)),
                (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_0001], Ok(536_870_911)),
                (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_1111], Ok(4_294_967_295)),
                (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0001_1111], Err(InvalidVaru32)),
                (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0011_1111], Err(InvalidVaru32)),
                (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0111_1111], Err(InvalidVaru32)),
                (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111], Err(InvalidVaru32)),
                (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_0001], Err(InvalidVaru32)),
            ].iter() {
            let (buffer, expected_result) : &(Vec<u8>, Result<u32, BinaryReaderError>) = item;
            let mut reader = BinaryReader::new(buffer);
            let actual_result: Result<u32, BinaryReaderError> = reader.read_var_u32();
            assert_eq!(*expected_result, actual_result);
        }
    }
}
