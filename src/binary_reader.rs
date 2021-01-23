use std::convert::TryInto;
use crate::binary_reader::BinaryReaderError::{UnexpectedEof, BadVersion, BadMagicNumber, InvalidVaru32, InvalidElementTypeByte, InvalidLimitsByte, InvalidValueTypeByte, InvalidMutableByte, InvalidVars33};
use std::{result, str};
use crate::types::{TableType, Limits, MemoryType, GlobalType, ValueType, ElementType, TableIndex, FuncIndex, DataType, MemoryIndex};
use crate::types::ValueType::{I32, I64, F32, F64};
use crate::BranchTableReader;

const WASM_MAGIC_NUMBER: &[u8; 4] = b"\0asm";
const WASM_SUPPORTED_VERSION: u32 = 0x1;

pub type Result<T, E = BinaryReaderError> = result::Result<T, E>;

#[derive(PartialEq, Eq, Debug)]
pub enum BinaryReaderError {
    UnexpectedEof,
    BadVersion,
    BadMagicNumber,
    InvalidVaru32,
    InvalidVars32,
    InvalidVars33,
    InvalidUtf8,
    InvalidElementTypeByte,
    InvalidLimitsByte,
    InvalidValueTypeByte,
    InvalidMutableByte,
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

    //TODO:Review and fix
    pub fn read_var_u32(&mut self) -> Result<u32> {
        let mut result: u32 = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_u8()?;
            result |= ((byte & 0b0111_1111) as u32) << shift;
            // The fifth byte's 4 high bits must be zero
            if shift == 28 && (byte >> 4) != 0 {
                return Err(InvalidVaru32);
            }
            shift += 7;
            if byte & 0b1000_0000 == 0 {
                break;
            }
        }
        Ok(result)
    }

    //TODO:Review and fix
    pub fn read_var_s33(&mut self) -> Result<i64> {
        let mut result: i64 = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_u8()?;
            result |= ((byte & 0b0111_1111) as i64) << shift;
            if shift == 28 {
                let continuation_bit = (byte & 0b1000_0000) != 0;
                let sign_and_unused_bit = (byte << 1) as i8 >> 5;
                if continuation_bit || (sign_and_unused_bit != 0 && sign_and_unused_bit != -1) {
                    return Err(InvalidVars33);
                }
            }
            shift += 7;
            if byte & 0b1000_0000 == 0 {
                //copy the sign bit to all unused_bits
                //by first shifting left by unused_bits
                //which will place the sign bit at MSB position
                //and then shifting right by unused_bits
                //which will copy the MSB bit to all unused_bits
                let unused_bits = 64 - shift;
                result = (result << unused_bits) >> unused_bits;
                break;
            }
        }
        Ok(result)
    }

    //TODO:Review and fix
    pub fn read_var_i32(&mut self) -> Result<i32> {
        let mut result: i32 = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_u8()?;
            result |= ((byte & 0b0111_1111) as i32) << shift;
            if shift == 28 {
                let continuation_bit = (byte & 0b1000_0000) != 0;
                let sign_and_unused_bit = (byte << 1) as i8 >> 4;
                return if continuation_bit || (sign_and_unused_bit != 0 && sign_and_unused_bit != -1) {
                    Err(InvalidVaru32)
                } else {
                    Ok(result)
                }
            }
            if byte & 0b1000_0000 == 0 {
                //copy the sign bit to all unused_bits
                //by first shifting left by unused_bits
                //which will place the sign bit at MSB position
                //and then shifting right by unused_bits
                //which will copy the MSB bit to all unused_bits
                let unused_bits = 32 - shift;
                result = (result << unused_bits) >> unused_bits;
                break;
            }
            shift += 7;
        }
        Ok(result)
    }

    //TODO:Implement
    pub fn read_var_i64(&mut self) -> Result<i64> {
        Ok(0)
    }

    //TODO:Implement
    pub fn read_var_f32(&mut self) -> Result<f32> {
        Ok(0.0)
    }

    //TODO:Implement
    pub fn read_var_f64(&mut self) -> Result<f64> {
        Ok(0.0)
    }

    pub fn create_branch_table_reader(&mut self) -> Result<BranchTableReader> {
        BranchTableReader::new(self.buffer)
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

    pub fn read_global_type(&mut self) -> Result<GlobalType> {
        let tp = self.read_value_type()?;
        let mutable = self.read_mutable_byte()?;
        Ok(GlobalType { var_type: tp, mutable })
    }

    fn read_mutable_byte(&mut self) -> Result<bool> {
        match self.read_u8()? {
            0x00 => Ok(false),
            0x01 => Ok(true),
            _ => Err(InvalidMutableByte),
        }
    }

    pub(crate) fn read_value_type(&mut self) -> Result<ValueType> {
        match self.read_u8()? {
            0x7F => Ok(I32),
            0xFE => Ok(I64),
            0x7D => Ok(F32),
            0x7C => Ok(F64),
            _ => Err(InvalidValueTypeByte)
        }
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

    pub fn read_element_type(&mut self) -> Result<ElementType> {
        let table_index = TableIndex(self.read_var_u32()?);
        //TODO:Not reading the expression for now
        loop {
            match self.read_u8()? {
                0x0B => break,
                _ => continue,
            }
        }
        let len = self.read_var_u32()?;
        let mut func_indices = Vec::with_capacity(len as usize);
        for _ in 0..len {
            let func_index = FuncIndex(self.read_var_u32()?);
            func_indices.push(func_index);
        }
        Ok(ElementType { table_index, function_indices: func_indices.into_boxed_slice() })
    }

    pub fn read_data_type(&mut self) -> Result<DataType> {
        let memory_index = MemoryIndex(self.read_var_u32()?);
        //TODO:Not reading the expression for now
        loop {
            match self.read_u8()? {
                0x0B => break,
                _ => continue,
            }
        }
        let len = self.read_var_u32()?;
        let bytes = &self.buffer[self.position..self.position + len as usize];
        Ok(DataType { memory_index, bytes })
    }
}

#[cfg(test)]
mod tests {
    use crate::binary_reader::{BinaryReader, BinaryReaderError};
    use crate::binary_reader::BinaryReaderError::{UnexpectedEof, InvalidVaru32, InvalidVars33};

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

    #[test]
    fn read_var_s33() {
        for item in
        [
            (vec![0b0000_0000], Ok(0i64)),
            (vec![0b0000_0001], Ok(1)),
            (vec![0b0000_0100], Ok(4)),
            (vec![0b0111_1111], Ok(-1)),
            (vec![0b1111_1111], Err(UnexpectedEof)),
            (vec![0b1111_1111, 0b0000_0000], Ok(127)),
            (vec![0b1111_1111, 0b0000_0001], Ok(255)),
            (vec![0b1111_1111, 0b0111_1111], Ok(-1)),
            (vec![0b1111_1111, 0b1111_1111, 0b0000_0001], Ok(32_767)),
            (vec![0b1111_1111, 0b1111_1111, 0b0111_1111], Ok(-1)),
            (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_0001], Ok(4_194_303)),
            (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0111_1111], Ok(-1)),
            (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_0001], Ok(536_870_911)),
            (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_1111], Ok(4_294_967_295)),
            (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0111_1111], Ok(-1)),
            (vec![0b1000_0000, 0b1000_0000, 0b1000_0000, 0b1000_0000, 0b0111_0000], Ok(-4_294_967_296)),
            (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0011_1111], Err(InvalidVars33)),
            (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111], Err(InvalidVars33)),
            (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_0001], Err(InvalidVars33)),
        ].iter() {
            let (buffer, expected_result) : &(Vec<u8>, Result<i64, BinaryReaderError>) = item;
            let mut reader = BinaryReader::new(buffer);
            let actual_result: Result<i64, BinaryReaderError> = reader.read_var_s33();
            assert_eq!(*expected_result, actual_result);
        }
    }

    #[test]
    fn read_var_s32() {
        for item in
        [
            // (vec![0b0000_0000], Ok(0i32)),
            // (vec![0b0000_0001], Ok(1)),
            // (vec![0b0000_0100], Ok(4)),
            // (vec![0b0111_1111], Ok(-1)),
            // (vec![0b1111_1111], Err(UnexpectedEof)),
            // (vec![0b1111_1111, 0b0000_0000], Ok(127)),
            // (vec![0b1111_1111, 0b0000_0001], Ok(255)),
            // (vec![0b1111_1111, 0b0111_1111], Ok(-1)),
            // (vec![0b1111_1111, 0b1111_1111, 0b0000_0001], Ok(32_767)),
            // (vec![0b1111_1111, 0b1111_1111, 0b0111_1111], Ok(-1)),
            // (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_0001], Ok(4_194_303)),
            // (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0111_1111], Ok(-1)),
            (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_0001], Ok(536_870_911)),
            // (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_1111], Ok(4_294_967_295)),
            // (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0111_1111], Ok(-1)),
            // (vec![0b1000_0000, 0b1000_0000, 0b1000_0000, 0b1000_0000, 0b0111_0000], Ok(-4_294_967_296)),
            // (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0011_1111], Err(InvalidVaru32)),
            // (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111], Err(InvalidVaru32)),
            // (vec![0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_0001], Err(InvalidVaru32)),
        ].iter() {
            let (buffer, expected_result) : &(Vec<u8>, Result<i32, BinaryReaderError>) = item;
            let mut reader = BinaryReader::new(buffer);
            let actual_result: Result<i32, BinaryReaderError> = reader.read_var_s32();
            assert_eq!(*expected_result, actual_result);
        }
    }
}
