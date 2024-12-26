use std::fs;

use super::opcode::Opcode;

const MEMORY_MAX: usize = 1 << 16;
const TOTAL_REGISTERS: usize = 8;

pub struct VM {
    registers: [u16; TOTAL_REGISTERS],
    pc: u16,
    cond: ConditionFlag,
    running: bool,
    memory: [u16; MEMORY_MAX],
}

#[derive(Debug)]
pub enum VMError {
    ReadingFile(String),
    ConcatenatingBytes(String),
    Adding(String),
    ReadingIndex(String),
    InvalidOpcode,
    InvalidRegister,
}

#[derive(Debug, PartialEq)]
enum ConditionFlag {
    Pos, // = 1 << 0, /* P */
    Zro, // = 1 << 1, /* Z */
    Neg, // = 1 << 2, /* N */
}

impl VM {
    pub fn new() -> VM {
        VM {
            registers: [0; TOTAL_REGISTERS],
            pc: 0x3000,
            cond: ConditionFlag::Zro,
            running: false,
            memory: [0; MEMORY_MAX],
        }
    }

    fn get_register(&self, register_index: u16) -> Result<&u16, VMError> {
        let register_index: usize = register_index.into();
        self.registers
            .get(register_index)
            .ok_or(VMError::InvalidRegister)
    }

    fn set_register(&mut self, store_register: u16, value: u16) -> Result<(), VMError> {
        let store_register: usize = store_register.into();
        *self
            .registers
            .get_mut(store_register)
            .ok_or(VMError::InvalidRegister)? = value;
        Ok(())
    }

    fn update_flags(&mut self, register: u16) -> Result<(), VMError> {
        let value = *self.get_register(register)?;
        if value == 0x0 {
            self.cond = ConditionFlag::Zro;
        } else if value >> 15 == 1 {
            self.cond = ConditionFlag::Neg;
        } else {
            self.cond = ConditionFlag::Pos;
        }
        Ok(())
    }

    pub fn read_image(&mut self, image_path: &str) -> Result<(), VMError> {
        let content = &fs::read(image_path).map_err(|e| {
            VMError::ReadingFile(format!("Failed to read file {}: {}", image_path, e))
        })?;
        self.read_image_bytes(content)?;
        Ok(())
    }

    fn read_image_bytes(&mut self, bytes: &[u8]) -> Result<(), VMError> {
        let mut collected: Vec<u16> = Vec::new();
        let mut chunks_of_two_bytes = bytes.chunks_exact(2);
        let origin: usize = Self::concatenate_bytes(chunks_of_two_bytes.next().ok_or(
            VMError::ConcatenatingBytes(String::from("No valid origin position from image")),
        )?)?
        .into();
        for chunk in chunks_of_two_bytes {
            let concatenated = Self::concatenate_bytes(chunk)?;
            collected.push(concatenated);
        }

        for (i, word) in collected.iter().enumerate() {
            let index = i.checked_add(origin).ok_or(VMError::Adding(String::from(
                "Invalid index accessing memory",
            )))?;
            let value = self
                .memory
                .get_mut(index)
                .ok_or(VMError::ReadingFile(String::from(
                    "Image exceeds memory capacity",
                )))?;
            *value = *word;
        }

        Ok(())
    }

    fn concatenate_bytes(bytes: &[u8]) -> Result<u16, VMError> {
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

    pub fn run(&mut self) -> Result<(), VMError> {
        while self.running {
            // Fetch
            let instr = self.mem_read(self.pc.into())?;
            self.pc = self
                .pc
                .checked_add(1)
                .ok_or(VMError::Adding(String::from("PC out of bounds")))?;
            let op: Opcode = (instr >> 12).try_into()?;

            match op {
                Opcode::BR => todo!(),
                Opcode::Add => self.add(instr)?,
                Opcode::LD => todo!(),
                Opcode::ST => todo!(),
                Opcode::Jsr => todo!(),
                Opcode::And => todo!(),
                Opcode::Ldr => todo!(),
                Opcode::Str => todo!(),
                Opcode::Rti => todo!(),
                Opcode::Not => todo!(),
                Opcode::Ldi => todo!(),
                Opcode::Sti => todo!(),
                Opcode::Jmp => todo!(),
                Opcode::Res => todo!(),
                Opcode::Lea => todo!(),
                Opcode::Trap => todo!(),
            }
        }

        Ok(())
    }

    fn mem_read(&self, index: usize) -> Result<u16, VMError> {
        let value = self
            .memory
            .get(index)
            .ok_or(VMError::ReadingIndex(String::from("Invalid index")))?;
        Ok(*value)
    }

    fn add(&mut self, instr: u16) -> Result<(), VMError> {
        let immediate_flag = (instr >> 5) & 0x1;
        let r0 = (instr >> 9) & 0x7;
        let r1 = (instr >> 6) & 0x7;
        let value_in_r1 = *self.get_register(r1)?;
        if immediate_flag == 1 {
            let imm5 = sign_extend(instr & 0b11111, 5)?;
            let sum = value_in_r1.wrapping_add(imm5);
            self.set_register(r0, sum)?;
        } else {
            let r2 = instr & 0x7;
            let value_in_r2 = *self.get_register(r2)?;
            let sum = value_in_r1.wrapping_add(value_in_r2);
            self.set_register(r0, sum)?;
        }
        self.update_flags(r0)?;
        Ok(())
    }
}

fn sign_extend(mut value: u16, bit_count: u16) -> Result<u16, VMError> {
    let last_bit_position = bit_count
        .checked_sub(1)
        .ok_or(VMError::Adding(String::from("Invalid last position bit")))?;
    if (value >> (last_bit_position) & 1) == 1 {
        value |= 0xFFFF << bit_count;
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_image_read() -> Result<(), VMError> {
        let mut vm = VM::new();
        // file containing 0x00 0x30 0xf2 0xf3 0xf4 0xf5 0xf6 0xf7
        vm.read_image("images/test-image-load-big-endian")?;
        assert_eq!(vm.memory[0x3000], 0xf3f2);
        assert_eq!(vm.memory[0x3001], 0xf5f4);
        assert_eq!(vm.memory[0x3002], 0xf7f6);
        Ok(())
    }

    #[test]
    fn sign_extend_mantains_sign() -> Result<(), VMError> {
        let x = 0b11111;
        let y = 0b01111;
        assert_eq!(sign_extend(x, 5)?, 0xFFFF);
        assert_eq!(sign_extend(y, 5)?, 0x000F);
        Ok(())
    }

    #[test]
    fn add_to_registers_and_store() -> Result<(), VMError> {
        let instr: u16 = 0b0001_0000_0100_0010;
        let mut vm = VM::new();
        vm.registers[1] = 1;
        vm.registers[2] = 3;
        vm.add(instr)?;
        assert_eq!(vm.registers[0], 4);
        assert_eq!(vm.cond, ConditionFlag::Pos);
        Ok(())
    }

    #[test]
    fn add_to_negative_value() -> Result<(), VMError> {
        let instr: u16 = 0b0001_0000_0100_0010;
        let mut vm = VM::new();
        vm.registers[1] = 1;
        vm.registers[2] = 0xFFFE; // -2
        vm.add(instr)?;
        assert_eq!(vm.registers[0], 0xFFFF); // -1
        assert_eq!(vm.cond, ConditionFlag::Neg);
        Ok(())
    }

    #[test]
    fn add_overflows() -> Result<(), VMError> {
        let instr: u16 = 0b0001_0000_0100_0010;
        let mut vm = VM::new();
        vm.registers[1] = 1;
        vm.registers[2] = 0xFFFF;
        vm.add(instr)?;
        assert_eq!(vm.registers[0], 0x0);
        assert_eq!(vm.cond, ConditionFlag::Zro);
        Ok(())
    }
}
