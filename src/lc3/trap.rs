use super::vm::VMError;

pub enum TrapCode {
    Getc,
    Out,
    Puts,
    IN,
    Putsp,
    Halt,
}

impl TryFrom<u16> for TrapCode {
    type Error = VMError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let trap = match value {
            0x20 => TrapCode::Getc,
            0x21 => TrapCode::Out,
            0x22 => TrapCode::Puts,
            0x23 => TrapCode::IN,
            0x24 => TrapCode::Putsp,
            0x25 => TrapCode::Halt,
            _ => return Err(VMError::InvalidTrapCode),
        };
        Ok(trap)
    }
}
