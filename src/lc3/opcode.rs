use std::convert::TryFrom;

use super::vm::VMError;
#[derive(Debug)]
pub enum Opcode {
    BR = 0, /* branch */
    Add,    /* add  */
    LD,     /* load */
    ST,     /* store */
    Jsr,    /* jump register */
    And,    /* bitwise and */
    Ldr,    /* load register */
    Str,    /* store register */
    Rti,    /* unused */
    Not,    /* bitwise not */
    Ldi,    /* load indirect */
    Sti,    /* store indirect */
    Jmp,    /* jump */
    Res,    /* reserved (unused) */
    Lea,    /* load effective address */
    Trap,   /* execute trap */
}

impl TryFrom<u16> for Opcode {
    type Error = VMError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0b0000 => Ok(Opcode::BR),
            0b0001 => Ok(Opcode::Add),
            0b0010 => Ok(Opcode::LD),
            0b0011 => Ok(Opcode::ST),
            0b0100 => Ok(Opcode::Jsr),
            0b0101 => Ok(Opcode::And),
            0b0110 => Ok(Opcode::Ldr),
            0b0111 => Ok(Opcode::Str),
            0b1000 => Ok(Opcode::Rti),
            0b1001 => Ok(Opcode::Not),
            0b1010 => Ok(Opcode::Ldi),
            0b1011 => Ok(Opcode::Sti),
            0b1100 => Ok(Opcode::Jmp),
            0b1101 => Ok(Opcode::Res),
            0b1110 => Ok(Opcode::Lea),
            0b1111 => Ok(Opcode::Trap),
            _ => Err(VMError::InvalidOpcode),
        }
    }
}
