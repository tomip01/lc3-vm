use super::vm::VMError;

pub fn sign_extend(mut value: u16, bit_count: u16) -> Result<u16, VMError> {
    let last_bit_position = bit_count.checked_sub(1).ok_or(VMError::Overflow)?;
    if (value >> (last_bit_position) & 1) == 1 {
        value |= 0xFFFF << bit_count;
    }
    Ok(value)
}

pub fn concatenate_bytes(bytes: &[u8]) -> Result<u16, VMError> {
    // maybe this check is redundant
    if bytes.len() != 2 {
        return Err(VMError::ConcatenatingBytes(String::from(
            "Image file is not made from words of 16 bits",
        )));
    }
    let first_byte: u8 = *bytes
        .first()
        .ok_or(VMError::ConcatenatingBytes(String::from(
            "Non existing first byte",
        )))?;
    let second_byte: u8 = *bytes
        .get(1)
        .ok_or(VMError::ConcatenatingBytes(String::from(
            "Non existing second byte",
        )))?;
    let res = u16::from_be_bytes([first_byte, second_byte]);
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_extend_mantains_sign() -> Result<(), VMError> {
        let x = 0b11111;
        let y = 0b01111;
        assert_eq!(sign_extend(x, 5)?, 0xFFFF);
        assert_eq!(sign_extend(y, 5)?, 0x000F);
        Ok(())
    }

    #[test]
    fn concatenates_two_bytes() -> Result<(), VMError> {
        let buffer: [u8; 2] = [0x12, 0x34];
        let concatenated = concatenate_bytes(&buffer)?;
        assert_eq!(concatenated, 0x1234);
        Ok(())
    }
}
