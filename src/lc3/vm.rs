use super::{memory::Memory, opcode::Opcode};

const TOTAL_REGISTERS: usize = 8;

pub struct VM {
    pub registers: [u16; TOTAL_REGISTERS],
    pub pc: u16,
    pub cond: ConditionFlag,
    pub running: bool,
    pub memory: Memory,
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
pub enum ConditionFlag {
    Pos,
    Zro,
    Neg,
}

impl VM {
    pub fn new() -> VM {
        VM {
            registers: [0; TOTAL_REGISTERS],
            pc: 0x3000,
            cond: ConditionFlag::Zro,
            running: false,
            memory: Memory::new(),
        }
    }

    pub fn get_register(&self, register_index: u16) -> Result<&u16, VMError> {
        let register_index: usize = register_index.into();
        self.registers
            .get(register_index)
            .ok_or(VMError::InvalidRegister)
    }

    pub fn set_register(&mut self, register_index: u16, value: u16) -> Result<(), VMError> {
        let store_register: usize = register_index.into();
        *self
            .registers
            .get_mut(store_register)
            .ok_or(VMError::InvalidRegister)? = value;
        Ok(())
    }

    pub fn update_flags(&mut self, register: u16) -> Result<(), VMError> {
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
        self.memory.read_image(image_path)
    }

    pub fn mem_read(&self, index: usize) -> Result<u16, VMError> {
        self.memory.mem_read(index)
    }

    pub fn mem_write(&mut self, value: u16, index: usize) -> Result<(), VMError> {
        self.memory.mem_write(value, index)
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

    pub fn execute(&mut self, instr: u16) -> Result<(), VMError> {
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
        }
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
        assert_eq!(vm.mem_read(0x3000)?, 0xf3f2);
        assert_eq!(vm.mem_read(0x3001)?, 0xf5f4);
        assert_eq!(vm.mem_read(0x3002)?, 0xf7f6);
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
        vm.mem_write(0x4242, 0xABCE)?;
        vm.mem_write(0x5353, 0xCDCF)?;
        vm.mem_write(0x3000, 0xABEF)?;
        for instr in instructions {
            vm.execute(instr)?;
        }
        assert_eq!(vm.pc, 0xABCD);
        assert_eq!(vm.registers[0], 0x0008);
        assert_eq!(vm.registers[6], 0x5353);
        assert_eq!(vm.registers[7], 0x4242);
        assert_eq!(vm.memory.mem_read(0x3000)?, 0x5353);
        Ok(())
    }
}
