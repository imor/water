use std::convert::TryInto;
use crate::binary_reader::BinaryReaderError::{UnexpectedEof, BadVersion, BadMagicNumber, InvalidU32, InvalidElementTypeByte, InvalidLimitsByte, InvalidValueTypeByte, InvalidMutableByte, InvalidS33, InvalidS32, InvalidS64};
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
    InvalidU32,
    InvalidS32,
    InvalidS64,
    InvalidS33,
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

    fn read_double_word(&mut self) -> Result<u32> {
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
            let version = self.read_double_word()?;
            if version == WASM_SUPPORTED_VERSION {
                Ok((self.position, version))
            } else {
                Err(BadVersion)
            }
        } else {
            Err(BadMagicNumber)
        }
    }

    pub fn read_byte(&mut self) -> Result<u8> {
        let bytes = self.read_bytes(1)?;
        Ok(bytes[0])
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let mut result: u32 = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_byte()?;
            result |= ((byte & 0b0111_1111) as u32) << shift;
            // The fifth byte's 4 high bits must be zero
            if shift == 28 && (byte >> 4) != 0 {
                return Err(InvalidU32);
            }
            shift += 7;
            if byte & 0b1000_0000 == 0 {
                break;
            }
        }
        Ok(result)
    }

    pub fn read_s33(&mut self) -> Result<i64> {
        let mut result: i64 = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_byte()?;
            result |= ((byte & 0b0111_1111) as i64) << shift;
            if shift == 28 {
                let more = (byte & 0b1000_0000) != 0;
                let sign_and_unused_bits = (byte << 1) as i8 >> 5;
                return if more || (sign_and_unused_bits != 0 && sign_and_unused_bits != -1) {
                    Err(InvalidS33)
                } else {
                    //extend the sign bit to all the unused bits
                    let unused_bits = 64 - 33;
                    result = (result << unused_bits) >> unused_bits;
                    Ok(result)
                }
            }
            shift += 7;
            if byte & 0b1000_0000 == 0 {
                //extend the sign bit to all unused_bits
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

    pub fn read_s32(&mut self) -> Result<i32> {
        let mut result: i32 = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_byte()?;
            result |= ((byte & 0b0111_1111) as i32) << shift;
            if shift == 28 {
                let more = (byte & 0b1000_0000) != 0;
                let sign_and_unused_bits = (byte << 1) as i8 >> 4;
                return if more || (sign_and_unused_bits != 0 && sign_and_unused_bits != -1) {
                    Err(InvalidS32)
                } else {
                    Ok(result)
                }
            }
            shift += 7;
            if byte & 0b1000_0000 == 0 {
                //extend the sign bit to all unused_bits
                //by first shifting left by unused_bits
                //which will place the sign bit at MSB position
                //and then shifting right by unused_bits
                //which will copy the MSB bit to all unused_bits
                let unused_bits = 32 - shift;
                result = (result << unused_bits) >> unused_bits;
                break;
            }
        }
        Ok(result)
    }

    pub fn read_s64(&mut self) -> Result<i64> {
        let mut result: i64 = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_byte()?;
            result |= ((byte & 0b0111_1111) as i64) << shift;
            if shift == 63 {
                let more = (byte & 0b1000_0000) != 0;
                let sign_and_unused_bits = (byte << 1) as i8 >> 1;
                return if more || (sign_and_unused_bits != 0 && sign_and_unused_bits != -1) {
                    Err(InvalidS64)
                } else {
                    Ok(result)
                }
            }
            shift += 7;
            if byte & 0b1000_0000 == 0 {
                //extend the sign bit to all unused_bits
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

    //TODO:Implement
    pub fn read_f32(&mut self) -> Result<f32> {
        Ok(0.0)
    }

    //TODO:Implement
    pub fn read_f64(&mut self) -> Result<f64> {
        Ok(0.0)
    }

    pub fn create_branch_table_reader(&mut self) -> Result<BranchTableReader> {
        BranchTableReader::new(self.buffer)
    }

    pub fn read_string(&mut self) -> Result<&'a str> {
        let len = self.read_u32()? as usize;
        let bytes = self.read_bytes(len)?;
        str::from_utf8(bytes).map_err(|_| BinaryReaderError::InvalidUtf8)
    }

    pub fn read_table_type(&mut self) -> Result<TableType> {
        match self.read_byte()? {
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
        match self.read_byte()? {
            0x00 => Ok(false),
            0x01 => Ok(true),
            _ => Err(InvalidMutableByte),
        }
    }

    pub(crate) fn read_value_type(&mut self) -> Result<ValueType> {
        match self.read_byte()? {
            0x7F => Ok(I32),
            0xFE => Ok(I64),
            0x7D => Ok(F32),
            0x7C => Ok(F64),
            _ => Err(InvalidValueTypeByte)
        }
    }

    fn read_limits(&mut self) -> Result<Limits> {
        match self.read_byte()? {
            0x00 => {
                let min = self.read_u32()?;
                let max = None;
                Ok(Limits { min, max })
            },
            0x01 => {
                let min = self.read_u32()?;
                let max = Some(self.read_u32()?);
                Ok(Limits { min, max })
            },
            _ => Err(InvalidLimitsByte)
        }
    }

    pub fn read_element_type(&mut self) -> Result<ElementType> {
        let table_index = TableIndex(self.read_u32()?);
        //TODO:Not reading the expression for now
        loop {
            match self.read_byte()? {
                0x0B => break,
                _ => continue,
            }
        }
        let len = self.read_u32()?;
        let mut func_indices = Vec::with_capacity(len as usize);
        for _ in 0..len {
            let func_index = FuncIndex(self.read_u32()?);
            func_indices.push(func_index);
        }
        Ok(ElementType { table_index, function_indices: func_indices.into_boxed_slice() })
    }

    pub fn read_data_type(&mut self) -> Result<DataType> {
        let memory_index = MemoryIndex(self.read_u32()?);
        //TODO:Not reading the expression for now
        loop {
            match self.read_byte()? {
                0x0B => break,
                _ => continue,
            }
        }
        let len = self.read_u32()?;
        let bytes = &self.buffer[self.position..self.position + len as usize];
        Ok(DataType { memory_index, bytes })
    }
}

#[cfg(test)]
mod tests {
    use crate::binary_reader::{BinaryReader, BinaryReaderError};
    use crate::binary_reader::BinaryReaderError::{UnexpectedEof, InvalidU32};

    fn encode_u32(mut num: u32) -> Vec<u8> {
        let mut result = Vec::new();
        loop {
            let mut byte = num as u8 & 0b0111_1111;
            num >>= 7;
            if num != 0 {
                byte |= 0b1000_0000;
            }
            result.push(byte);
            if num == 0 {
                break;
            }
        }
        result
    }

    //Ignoring this test because it takes almost an hour to run
    #[ignore]
    #[test]
    fn u32_roundtrip() {
        let lot_size = 10000000;
        let mut lot = 1;
        for i in 0..=u32::max_value() {
            let encoded = encode_u32(i);
            let mut reader = BinaryReader::new(&encoded);
            let actual_result: Result<u32, BinaryReaderError> = reader.read_u32();
            assert_eq!(Ok(i), actual_result);
            if i % lot_size == 0 {
                println!("Done {} lots of {}", lot, u32::max_value() / lot_size);
                lot += 1;
            }
        }
    }

    //Ignoring this test because it takes almost an hour to run
    #[ignore]
    #[test]
    fn invalid_more_bit_u32() {
        let lot_size = 10000000;
        let mut lot = 1;
        let total = u32::max_value() - 268_435_456;
        for i in 268_435_456..=u32::max_value() {
            let mut encoded = encode_u32(i);
            assert_eq!(5, encoded.len());
            let mut last_byte = encoded[4];
            last_byte |= 0b1000_0000;
            encoded[4] = last_byte;
            let mut reader = BinaryReader::new(&encoded);
            let actual_result: Result<u32, BinaryReaderError> = reader.read_u32();
            assert_eq!(Err(InvalidU32), actual_result);
            if i % lot_size == 0 {
                println!("Done {} lots of {}", lot, total / lot_size);
                lot += 1;
            }
        }
    }

    fn encode_s32(mut num: i32) -> Vec<u8> {
        let mut result = Vec::new();
        let mut more = true;
        loop {
            let mut byte = num as u8 & 0b0111_1111;
            num >>= 7;
            if (num == 0 && byte & 0b0100_0000 == 0) || (num == -1 && byte & 0b0100_0000 == 0b0100_0000) {
                more = false;
            } else {
                byte |= 0b1000_0000;
            }
            result.push(byte);
            if !more {
                break;
            }
        }
        result
    }

    //Ignoring this test because it takes almost an hour to run
    #[ignore]
    #[test]
    fn s32_roundtrip() {
        let lot_size = 10000000;
        let mut lot = 1;
        for i in i32::min_value()..=i32::max_value() {
            let encoded = encode_s32(i);
            let mut reader = BinaryReader::new(&encoded);
            let actual_result: Result<i32, BinaryReaderError> = reader.read_s32();
            assert_eq!(Ok(i), actual_result);
            if i % lot_size == 0 {
                println!("Done {} lots of {}", lot, u32::max_value() / lot_size as u32);
                lot += 1;
            }
        }
    }

    fn encode_s33(mut num: i64) -> Vec<u8> {
        let mut result = Vec::new();
        let mut more = true;
        loop {
            let mut byte = num as u8 & 0b0111_1111;
            num >>= 7;
            if (num == 0 && byte & 0b0100_0000 == 0) || (num == -1 && byte & 0b0100_0000 == 0b0100_0000) {
                more = false;
            } else {
                byte |= 0b1000_0000;
            }
            result.push(byte);
            if !more {
                break;
            }
        }
        result
    }

    //Ignoring this test because it takes almost two hours to run
    #[ignore]
    #[test]
    fn s33_roundtrip() {
        let lot_size = 10000000;
        let mut lot = 1;
        let min: i64 = -4_294_967_296;
        let max: i64 = 4_294_967_296;
        for i in min..max {
            let encoded = encode_s33(i);
            let mut reader = BinaryReader::new(&encoded);
            let actual_result: Result<i64, BinaryReaderError> = reader.read_s33();
            assert_eq!(Ok(i), actual_result);
            if i % lot_size == 0 {
                println!("Done {} lots of {}", lot, 2 * max / lot_size);
                lot += 1;
            }
        }
    }

    fn encode_s64(num: i64) -> Vec<u8> {
        encode_s33(num)
    }

    #[test]
    fn s64_roundtrip() {
        let mut lot = 1;
        let lot_size = 1000000;
        let ranges = vec![
            -9_223_372_036_854_775_808..-9_223_372_036_854_775_808+lot_size,
            -lot_size..lot_size,
            (9_223_372_036_854_775_807-lot_size)..9_223_372_036_854_775_807,
        ];
        let total = ranges.iter().fold(0, |acc, x| acc +(x.end - x.start));
        for r in ranges {
            for i in r {
                let encoded = encode_s64(i);
                let mut reader = BinaryReader::new(&encoded);
                let actual_result: Result<i64, BinaryReaderError> = reader.read_s64();
                assert_eq!(Ok(i), actual_result);
                if i % lot_size == 0 {
                    println!("Done {} lots of {}", lot, total / lot_size);
                    lot += 1;
                }
            }
        }
    }
}
