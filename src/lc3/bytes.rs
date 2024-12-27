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
