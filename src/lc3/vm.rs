use std::fs;

use super::{
    bytes::{concatenate_bytes, sign_extend},
    opcode::Opcode,
};

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
#[allow(dead_code)]
pub enum VMError {
    ReadingFile(String),
    ConcatenatingBytes(String),
    Overflow,
    MemoryIndex(String),
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

    fn set_register(&mut self, register_index: u16, value: u16) -> Result<(), VMError> {
        let store_register: usize = register_index.into();
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
        let origin: usize = concatenate_bytes(chunks_of_two_bytes.next().ok_or(
            VMError::ConcatenatingBytes(String::from("No valid origin position from image")),
        )?)?
        .into();
        for chunk in chunks_of_two_bytes {
            let concatenated = concatenate_bytes(chunk)?;
            collected.push(concatenated);
        }

        for (i, word) in collected.iter().enumerate() {
            let index = i
                .checked_add(origin)
                .ok_or(VMError::MemoryIndex(String::from(
                    "Invalid index to access memory on loading",
                )))?;
            let value = self
                .memory
                .get_mut(index)
                .ok_or(VMError::MemoryIndex(String::from(
                    "Image exceeds memory capacity",
                )))?;
            *value = *word;
        }

        Ok(())
    }

    fn mem_read(&self, index: usize) -> Result<u16, VMError> {
        let value = self
            .memory
            .get(index)
            .ok_or(VMError::MemoryIndex(String::from(
                "Invalid out of bounds when reading from memory",
            )))?;
        Ok(*value)
    }

    fn mem_write(&mut self, value: u16, index: usize) -> Result<(), VMError> {
        let cell = self
            .memory
            .get_mut(index)
            .ok_or(VMError::MemoryIndex(String::from(
                "Index out of bound when writing memory",
            )))?;
        *cell = value;
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), VMError> {
        while self.running {
            // Fetch
            let instr = self.mem_read(self.pc.into())?;
            // Increment PC
            self.pc = self
                .pc
                .checked_add(1)
                .ok_or(VMError::MemoryIndex(String::from("PC out of bounds")))?;
            // Execute
            self.execute(instr)?;
        }

        Ok(())
    }

    fn execute(&mut self, instr: u16) -> Result<(), VMError> {
        let op: Opcode = (instr >> 12).try_into()?;
        match op {
            Opcode::BR => self.br(instr),
            Opcode::Add => self.add(instr),
            Opcode::LD => self.ld(instr),
            Opcode::ST => self.st(instr),
            Opcode::Jsr => self.jsr(instr),
            Opcode::And => self.and(instr),
            Opcode::Ldr => self.ldr(instr),
            Opcode::Str => self.str(instr),
            Opcode::Rti => Err(VMError::InvalidOpcode),
            Opcode::Not => self.not(instr),
            Opcode::Ldi => self.ldi(instr),
            Opcode::Sti => self.sti(instr),
            Opcode::Jmp => self.jmp(instr),
            Opcode::Res => Err(VMError::InvalidOpcode),
            Opcode::Lea => self.lea(instr),
            Opcode::Trap => todo!(),
        }?;
        Ok(())
    }

    fn add(&mut self, instr: u16) -> Result<(), VMError> {
        let immediate_flag = (instr >> 5) & 1;
        let r0 = (instr >> 9) & 0b0111;
        let r1 = (instr >> 6) & 0b0111;
        let value_in_r1 = *self.get_register(r1)?;
        if immediate_flag == 1 {
            let imm5 = sign_extend(instr & 0b0001_1111, 5)?;
            let sum = value_in_r1.wrapping_add(imm5);
            self.set_register(r0, sum)?;
        } else {
            let r2 = instr & 0b0111;
            let value_in_r2 = *self.get_register(r2)?;
            let sum = value_in_r1.wrapping_add(value_in_r2);
            self.set_register(r0, sum)?;
        }
        self.update_flags(r0)?;
        Ok(())
    }

    fn and(&mut self, instr: u16) -> Result<(), VMError> {
        let immediate_flag = (instr >> 5) & 1;
        let r0 = (instr >> 9) & 0b0111;
        let r1 = (instr >> 6) & 0b0111;
        let value_in_r1 = *self.get_register(r1)?;
        if immediate_flag == 1 {
            let imm5 = sign_extend(instr & 0b11111, 5)?;
            let res = value_in_r1 & imm5;
            self.set_register(r0, res)?;
        } else {
            let r2 = instr & 0b0111;
            let value_in_r2 = *self.get_register(r2)?;
            let res = value_in_r1 & value_in_r2;
            self.set_register(r0, res)?;
        }
        self.update_flags(r0)?;
        Ok(())
    }

    fn not(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0b0111;
        let r1 = (instr >> 6) & 0b0111;
        let value_in_r1 = *self.get_register(r1)?;
        self.set_register(r0, !value_in_r1)?;
        self.update_flags(r0)?;
        Ok(())
    }

    fn br(&mut self, instr: u16) -> Result<(), VMError> {
        let pc_offset = sign_extend(instr & 0b0001_1111_1111, 9)?;
        let cond_flag_instr = (instr >> 9) & 0b0111;
        let meet_condition = match self.cond {
            ConditionFlag::Neg => cond_flag_instr & 0b100,
            ConditionFlag::Zro => cond_flag_instr & 0b010,
            ConditionFlag::Pos => cond_flag_instr & 0b001,
        };
        if meet_condition != 0 {
            self.pc = self
                .pc
                .checked_add(pc_offset)
                .ok_or(VMError::MemoryIndex(String::from(
                    "Overflow with offset in conditional branching",
                )))?;
        }
        Ok(())
    }

    fn jmp(&mut self, instr: u16) -> Result<(), VMError> {
        let r1 = (instr >> 6) & 0b0111;
        let value_in_r1 = *self.get_register(r1)?;
        self.pc = value_in_r1;
        Ok(())
    }

    fn jsr(&mut self, instr: u16) -> Result<(), VMError> {
        let long_flag = (instr >> 11) & 1;
        self.set_register(7, self.pc)?;
        if long_flag == 1 {
            let long_pc_offset = sign_extend(instr & 0b0111_1111_1111, 11)?;
            self.pc = self
                .pc
                .checked_add(long_pc_offset)
                .ok_or(VMError::MemoryIndex(String::from(
                    "Overflow with offset in jumping register",
                )))?;
        } else {
            let r1 = (instr >> 6) & 0b0111;
            let value_in_r1 = *self.get_register(r1)?;
            self.pc = value_in_r1;
        }
        Ok(())
    }

    fn lea(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0b0111;
        let pc_offset = sign_extend(instr & 0b0001_1111_1111, 9)?;
        let value = self
            .pc
            .checked_add(pc_offset)
            .ok_or(VMError::MemoryIndex(String::from(
                "Overflow with offset in LEA",
            )))?;
        self.set_register(r0, value)?;
        self.update_flags(r0)?;
        Ok(())
    }

    fn ld(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0b0111;
        let pc_offset = sign_extend(instr & 0b0001_1111_1111, 9)?;
        let address = self
            .pc
            .checked_add(pc_offset)
            .ok_or(VMError::MemoryIndex(String::from(
                "Overflow with offset in Load",
            )))?;
        let value_read = self.mem_read(address.into())?;
        self.set_register(r0, value_read)?;
        self.update_flags(r0)?;
        Ok(())
    }

    fn ldr(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0b0111;
        let r1 = (instr >> 6) & 0b0111;
        let value_in_r1 = *self.get_register(r1)?;
        let pc_offset = sign_extend(instr & 0b0011_1111, 6)?;
        let address = value_in_r1
            .checked_add(pc_offset)
            .ok_or(VMError::MemoryIndex(String::from(
                "Overflow with offset in Load Register",
            )))?;
        let value_read = self.mem_read(address.into())?;
        self.set_register(r0, value_read)?;
        self.update_flags(r0)?;
        Ok(())
    }

    fn ldi(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0b0111;
        let pc_offset = sign_extend(instr & 0b0001_1111_1111, 9)?;
        let address = self
            .pc
            .checked_add(pc_offset)
            .ok_or(VMError::MemoryIndex(String::from(
                "Overflow with offset in Load Indirect",
            )))?;
        let value_read = self.mem_read(address.into())?;
        self.set_register(r0, self.mem_read(value_read.into())?)?;
        self.update_flags(r0)?;
        Ok(())
    }

    fn st(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0b0111;
        let value_in_r0 = *self.get_register(r0)?;
        let pc_offset = sign_extend(instr & 0b0001_1111_1111, 9)?;
        let address = self
            .pc
            .checked_add(pc_offset)
            .ok_or(VMError::MemoryIndex(String::from(
                "Overflow with offset in Store",
            )))?;
        self.mem_write(value_in_r0, address.into())?;
        Ok(())
    }

    fn sti(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0b0111;
        let value_in_r0 = *self.get_register(r0)?;
        let pc_offset = sign_extend(instr & 0b0001_1111_1111, 9)?;
        let address = self
            .pc
            .checked_add(pc_offset)
            .ok_or(VMError::MemoryIndex(String::from(
                "Overflow with offset in Store Indirect",
            )))?;
        let value_read = self.mem_read(address.into())?;
        self.mem_write(value_in_r0, value_read.into())?;
        Ok(())
    }

    fn str(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0b0111;
        let value_in_r0 = *self.get_register(r0)?;
        let r1 = (instr >> 6) & 0b0111;
        let value_in_r1 = *self.get_register(r1)?;
        let pc_offset = sign_extend(instr & 0b0011_1111, 6)?;
        let address = value_in_r1
            .checked_add(pc_offset)
            .ok_or(VMError::MemoryIndex(String::from(
                "Overflow with offset in Store Register",
            )))?;
        self.mem_write(value_in_r0, address.into())?;
        Ok(())
    }
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
    fn add_to_zero() -> Result<(), VMError> {
        let instr: u16 = 0b0001_0000_0100_0010;
        let mut vm = VM::new();
        vm.registers[1] = 1;
        vm.registers[2] = 0xFFFF;
        vm.add(instr)?;
        assert_eq!(vm.registers[0], 0x0);
        assert_eq!(vm.cond, ConditionFlag::Zro);
        Ok(())
    }

    #[test]
    fn add_with_immediate() -> Result<(), VMError> {
        let instr: u16 = 0b0001_0000_0110_0010; // add 2
        let mut vm = VM::new();
        vm.registers[1] = 1;
        vm.add(instr)?;
        assert_eq!(vm.registers[0], 0x3);
        assert_eq!(vm.cond, ConditionFlag::Pos);
        Ok(())
    }

    #[test]
    fn and_register() -> Result<(), VMError> {
        let instr: u16 = 0b0101_0000_0100_0010;
        let mut vm = VM::new();
        vm.registers[1] = 0x0F0F;
        vm.registers[2] = 0xFFFF;
        vm.and(instr)?;
        assert_eq!(vm.registers[0], 0x0F0F);
        assert_eq!(vm.cond, ConditionFlag::Pos);
        Ok(())
    }

    #[test]
    fn and_with_immediate() -> Result<(), VMError> {
        let instr: u16 = 0b0001_0000_0110_0111;
        let mut vm = VM::new();
        vm.registers[1] = 0xFFFF;
        vm.and(instr)?;
        assert_eq!(vm.registers[0], 0b0111);
        assert_eq!(vm.cond, ConditionFlag::Pos);
        Ok(())
    }

    #[test]
    fn not_register() -> Result<(), VMError> {
        let instr: u16 = 0b1001_0000_0111_1111;
        let mut vm = VM::new();
        vm.registers[1] = 0x0F0F;
        vm.not(instr)?;
        assert_eq!(vm.registers[0], 0xF0F0);
        assert_eq!(vm.cond, ConditionFlag::Neg);
        Ok(())
    }

    #[test]
    fn branch_on_flag() -> Result<(), VMError> {
        let mut vm = VM::new();
        vm.cond = ConditionFlag::Pos;
        let instr: u16 = 0b0000_0010_0000_1010; // jump 10 places on positive flag
        vm.br(instr)?;
        assert_eq!(vm.pc, 0x300A);
        Ok(())
    }

    #[test]
    fn dont_branch_on_flag() -> Result<(), VMError> {
        let mut vm = VM::new();
        vm.cond = ConditionFlag::Neg;
        let instr: u16 = 0b0000_0010_0000_1010; // jump 10 places on positive flag
        vm.br(instr)?;
        assert_eq!(vm.pc, 0x3000);
        Ok(())
    }

    #[test]
    fn jump_to_correct_value() -> Result<(), VMError> {
        let mut vm = VM::new();
        let instr: u16 = 0b1100_0000_0100_0000; // jump to value in R1
        vm.registers[1] = 0x4242;
        vm.jmp(instr)?;
        assert_eq!(vm.pc, 0x4242);
        Ok(())
    }

    #[test]
    fn jump_to_subroutine() -> Result<(), VMError> {
        let mut vm = VM::new();
        let instr: u16 = 0b0100_1000_0100_0010; // add to PC 0x42
        vm.jsr(instr)?;
        assert_eq!(vm.pc, 0x3042);
        Ok(())
    }

    #[test]
    fn jump_to_subroutine_register() -> Result<(), VMError> {
        let mut vm = VM::new();
        let instr: u16 = 0b0100_0000_1100_0000; // set PC to R3 value
        vm.registers[3] = 0x4242;
        vm.jsr(instr)?;
        assert_eq!(vm.pc, 0x4242);
        Ok(())
    }

    #[test]
    fn lea_store_address() -> Result<(), VMError> {
        let mut vm = VM::new();
        let instr: u16 = 0b1110_0000_0100_0010; // store in R0, PC + 0x42
        vm.lea(instr)?;
        assert_eq!(vm.registers[0], 0x3042);
        Ok(())
    }

    #[test]
    fn load_to_register() -> Result<(), VMError> {
        let mut vm = VM::new();
        let instr: u16 = 0b0010_0000_0100_0010; // load in R0, value stored in PC + 0x42
        vm.memory[0x3042] = 0x4242;
        vm.ld(instr)?;
        assert_eq!(vm.registers[0], 0x4242);
        Ok(())
    }

    #[test]
    fn load_register_to_register() -> Result<(), VMError> {
        let mut vm = VM::new();
        let instr: u16 = 0b0110_0000_0100_0010; // load in R0, value stored in R1 + 0x02
        vm.registers[1] = 0x3040;
        vm.memory[0x3042] = 0x4242;
        vm.ld(instr)?;
        assert_eq!(vm.registers[0], 0x4242);
        Ok(())
    }

    #[test]
    fn load_indirect() -> Result<(), VMError> {
        let mut vm = VM::new();
        let instr: u16 = 0b1010_0000_0100_0010; // pc_offset is 0x42, will look for address in 0x3042
        vm.memory[0x3042] = 0x4242;
        vm.memory[0x4242] = 0x5353;
        vm.ldi(instr)?;
        assert_eq!(vm.registers[0], 0x5353);
        Ok(())
    }

    #[test]
    fn store_value() -> Result<(), VMError> {
        let mut vm = VM::new();
        let instr: u16 = 0b0011_0000_0100_0010; // pc_offset is 0x42, store in 0x3042 what is in R0
        vm.registers[0] = 0x4242;
        vm.st(instr)?;
        assert_eq!(vm.memory[0x3042], 0x4242);
        Ok(())
    }

    #[test]
    fn store_indirect_value() -> Result<(), VMError> {
        let mut vm = VM::new();
        let instr: u16 = 0b1011_0000_0100_0010; // pc_offset is 0x42, store what is in R0 in [0x3042]
        vm.memory[0x3042] = 0x5353;
        vm.registers[0] = 0x4242;
        vm.sti(instr)?;
        assert_eq!(vm.memory[0x5353], 0x4242);
        Ok(())
    }

    #[test]
    fn store_register_value() -> Result<(), VMError> {
        let mut vm = VM::new();
        let instr: u16 = 0b0111_0000_0100_0010; // px_offset is 0x02 this plus what is in R1 is the address,
                                                //store there what is in R0
        vm.registers[0] = 0x4242;
        vm.registers[1] = 0x1234;
        vm.str(instr)?;
        assert_eq!(vm.memory[0x1236], 0x4242);
        Ok(())
    }

    #[test]
    fn combined_register_instructions() -> Result<(), VMError> {
        let mut vm = VM::new();
        let instructions = [
            0b0001_0000_0100_0010, // ADD R0, R1, R2        => R0 = 0x7978
            0b0101_0000_0010_1111, // AND R0, R0, 0x000F    => R0 = 0x0008
            0b0000_0010_0100_0010, // BR+ 0x0042            => PC = 0x3042
            0b1100_0000_0100_0000, // JMP R1                => PC = 0xABAB
            0b0100_1000_0010_0010, // JSR 0x0022            => PC = 0xABCD
            0b0010_1110_0000_0001, // LD R7, 0x1            => R7 = [0xABCE]
            0b0110_1100_1000_0010, // LDR R6, R2, 0x2       => R6 = [0xCDCF]
            0b1011_1100_0010_0010, // STI R6, 0x22          => [[0xABEF]] = R6
        ];
        vm.registers[1] = 0xABAB;
        vm.registers[2] = 0xCDCD;
        vm.memory[0xABCE] = 0x4242;
        vm.memory[0xCDCF] = 0x5353;
        vm.memory[0xABEF] = 0x3000;
        for instr in instructions {
            vm.execute(instr)?;
        }
        assert_eq!(vm.pc, 0xABCD);
        assert_eq!(vm.registers[0], 0x0008);
        assert_eq!(vm.registers[6], 0x5353);
        assert_eq!(vm.registers[7], 0x4242);
        assert_eq!(vm.memory[0x3000], 0x5353);
        Ok(())
    }
}
