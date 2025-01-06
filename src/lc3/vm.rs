use std::io::{stdin, stdout, Read, Write};

use super::{bytes::sign_extend, memory::Memory, opcode::Opcode, trap::TrapCode};

const TOTAL_REGISTERS: usize = 8;

pub struct VM {
    registers: [u16; TOTAL_REGISTERS],
    pc: u16,
    cond: ConditionFlag,
    running: bool,
    memory: Memory,
}

#[derive(Debug)]
pub enum VMError {
    ReadingFile(String),
    ConcatenatingBytes(String),
    Overflow,
    MemoryIndex(String),
    InvalidOpcode,
    InvalidRegister,
    InvalidTrapCode,
    StandardIO(String),
    InvalidCharacter,
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

    /// Get value stored in the register requested in `register_index`
    ///
    /// If `register_index` is outside of the range from 0 to 7, an Error is returned
    fn get_register(&self, register_index: u16) -> Result<&u16, VMError> {
        let register_index: usize = register_index.into();
        self.registers
            .get(register_index)
            .ok_or(VMError::InvalidRegister)
    }

    /// Set the value `value` in the register requested in `register_index`
    ///
    /// If `register_index` is outside of the range from 0 to 7, an Error is returned
    fn set_register(&mut self, register_index: u16, value: u16) -> Result<(), VMError> {
        let store_register: usize = register_index.into();
        *self
            .registers
            .get_mut(store_register)
            .ok_or(VMError::InvalidRegister)? = value;
        Ok(())
    }

    /// Update flags on `cond` member of the VM
    /// based on the content of the register requested in `register_index`.
    ///
    /// * If the content is a zero, then the flag Zro is stored in `cond`
    /// * If the content is a negative number (using two complement codification),
    ///   then the flag Neg is stored in `cond`
    /// * If the content is a positive number, then the flag Pos is stored in `cond`
    ///
    /// If `register_index` is outside of the range from 0 to 7, an Error is returned
    fn update_flags(&mut self, register_index: u16) -> Result<(), VMError> {
        let value = *self.get_register(register_index)?;
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

    fn mem_read(&mut self, index: usize) -> Result<u16, VMError> {
        self.memory.mem_read(index)
    }

    pub fn mem_write(&mut self, value: u16, index: usize) -> Result<(), VMError> {
        self.memory.mem_write(value, index)
    }

    /// Main execution loop
    ///
    /// Reads the instruction the PC points to, increments the PC and executes the read instruction
    /// Repeat this while the machine doesn't execute a trap instruction with the HALT routine (TRAP HALT)
    pub fn run(&mut self) -> Result<(), VMError> {
        // start machine
        self.running = true;

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

    /// The first 4 bits of the `instr` are casted as an Opcode
    /// Based on the Opcode it executes a specific instruction.
    ///
    /// The other 12 bits are interpreted differently according each instruction
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
            Opcode::Rti => Err(VMError::InvalidOpcode), // unused, requires Supervisor privileges, User cannot execute it
            Opcode::Not => self.not(instr),
            Opcode::Ldi => self.ldi(instr),
            Opcode::Sti => self.sti(instr),
            Opcode::Jmp => self.jmp(instr),
            Opcode::Res => Err(VMError::InvalidOpcode), // unused (reserved)
            Opcode::Lea => self.lea(instr),
            Opcode::Trap => self.trap(instr),
        }
    }

    /// ADD
    ///
    /// ADD DR, SR1, SR2 or ADD DR, SR1, imm5
    ///
    /// If Bit 5 is 0, stores in DR the addition between SR1 and SR2.
    /// If Bit 5 is 1, stores in DR the addition between SR1 and imm5 (sign extended).
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

    /// AND
    ///
    /// AND DR, SR1, SR2 or AND DR, SR1, imm5
    ///
    /// If Bit 5 is 0, stores in DR the and bitwise between SR1 and SR2.
    /// If Bit 5 is 1, stores in DR the and bitwise between SR1 and imm5 (sign extended).
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

    /// NOT
    ///
    /// NOT DR, SR1
    ///
    /// Stores in DR the negate bitwise of SR1
    fn not(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0b0111;
        let r1 = (instr >> 6) & 0b0111;
        let value_in_r1 = *self.get_register(r1)?;
        self.set_register(r0, !value_in_r1)?;
        self.update_flags(r0)?;
        Ok(())
    }

    /// BR
    ///
    /// BRnzp PCoffset9
    ///
    /// Bits 9, 10 and 11 are to set on which conditions make a jump if they match with the `cond` flag.
    ///
    /// If bit 9 is 1 and `cond` is Pos, then jump.
    /// If bit 10 is 1 and `cond` is Zro, then jump.
    /// If bit 11 is 1 and `cond` is Neg, then jump.
    ///
    /// Note that you can set multiple bits
    ///
    /// Stores in PC the addition of the PC and the sign extended PCoffset9 only if
    /// any of the conditions are met
    fn br(&mut self, instr: u16) -> Result<(), VMError> {
        let pc_offset = sign_extend(instr & 0b0001_1111_1111, 9)?;
        let cond_flag_instr = (instr >> 9) & 0b0111;
        let meet_condition = match self.cond {
            ConditionFlag::Neg => cond_flag_instr & 0b100,
            ConditionFlag::Zro => cond_flag_instr & 0b010,
            ConditionFlag::Pos => cond_flag_instr & 0b001,
        };
        if meet_condition != 0 {
            self.pc = self.pc.wrapping_add(pc_offset);
        }
        Ok(())
    }

    /// JMP
    ///
    /// JMP BaseR
    ///
    /// Stores in PC the addition of the PC and the value stored in the register BaseR
    fn jmp(&mut self, instr: u16) -> Result<(), VMError> {
        let r1 = (instr >> 6) & 0b0111;
        let value_in_r1 = *self.get_register(r1)?;
        self.pc = value_in_r1;
        Ok(())
    }

    /// JSR/JSRR
    ///
    /// JSR PCoffset11 or JSRR BaseR
    ///
    /// If bit 11 is a 1, then PC is added with the sign extension of PCoffset11
    /// Else, PC is added with the value in the register
    fn jsr(&mut self, instr: u16) -> Result<(), VMError> {
        let long_flag = (instr >> 11) & 1;
        self.set_register(7, self.pc)?;
        if long_flag == 1 {
            let long_pc_offset = sign_extend(instr & 0b0111_1111_1111, 11)?;
            self.pc = self.pc.wrapping_add(long_pc_offset);
        } else {
            let r1 = (instr >> 6) & 0b0111;
            let value_in_r1 = *self.get_register(r1)?;
            self.pc = value_in_r1;
        }
        Ok(())
    }

    /// LEA
    ///
    /// LEA DR, LABEL
    ///
    /// Stores in the register DR the value of the label
    fn lea(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0b0111;
        let pc_offset = sign_extend(instr & 0b0001_1111_1111, 9)?;
        let value = self.pc.wrapping_add(pc_offset);
        self.set_register(r0, value)?;
        self.update_flags(r0)?;
        Ok(())
    }

    /// LD
    ///
    /// LD DR, PCoffset9
    ///
    /// Loads the value in memory from the address that is obtained sign extending PCoffset9 plus the PC
    /// into the register DR
    fn ld(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0b0111;
        let pc_offset = sign_extend(instr & 0b0001_1111_1111, 9)?;
        let address = self.pc.wrapping_add(pc_offset);
        let value_read = self.mem_read(address.into())?;
        self.set_register(r0, value_read)?;
        self.update_flags(r0)?;
        Ok(())
    }

    /// LDR
    ///
    /// LDR DR, BaseR, offset6
    ///
    /// Stores in DR the value from memory that is obtained sign extending offset6 plus
    /// the value in the register BaseR
    fn ldr(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0b0111;
        let r1 = (instr >> 6) & 0b0111;
        let value_in_r1 = *self.get_register(r1)?;
        let pc_offset = sign_extend(instr & 0b0011_1111, 6)?;
        let address = value_in_r1.wrapping_add(pc_offset);
        let value_read = self.mem_read(address.into())?;
        self.set_register(r0, value_read)?;
        self.update_flags(r0)?;
        Ok(())
    }

    /// LDI
    ///
    /// LDI DR, PCoffset9
    ///
    /// Stores in the register DR the value in memory where the address is stored in the position in memory of
    /// the PC plus the sign extension of PCoffset9
    fn ldi(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0b0111;
        let pc_offset = sign_extend(instr & 0b0001_1111_1111, 9)?;
        let address = self.pc.wrapping_add(pc_offset);
        let value_read = self.mem_read(address.into())?;
        let value = self.mem_read(value_read.into())?;
        self.set_register(r0, value)?;
        self.update_flags(r0)?;
        Ok(())
    }

    /// ST
    ///
    /// ST SR, PCoffset9
    ///
    /// Stores in the address of the PC plus the sign extension of PCoffset9 the value in
    /// the register SR
    fn st(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0b0111;
        let value_in_r0 = *self.get_register(r0)?;
        let pc_offset = sign_extend(instr & 0b0001_1111_1111, 9)?;
        let address = self.pc.wrapping_add(pc_offset);
        self.mem_write(value_in_r0, address.into())?;
        Ok(())
    }

    /// STI
    ///
    /// STI SR, PCoffset9
    ///
    /// Stores in the address that is stored in memory in the position of the PC plus the sign extension of PCoffset9,
    /// the value in the register SR
    fn sti(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0b0111;
        let value_in_r0 = *self.get_register(r0)?;
        let pc_offset = sign_extend(instr & 0b0001_1111_1111, 9)?;
        let address = self.pc.wrapping_add(pc_offset);
        let value_read = self.mem_read(address.into())?;
        self.mem_write(value_in_r0, value_read.into())?;
        Ok(())
    }

    /// STR
    ///
    /// STR SR, BaseR, offset6
    ///
    /// Stores in the address of the BaseR plus the sign extension of PCoffset6 the value in
    /// the register SR
    fn str(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0b0111;
        let value_in_r0 = *self.get_register(r0)?;
        let r1 = (instr >> 6) & 0b0111;
        let value_in_r1 = *self.get_register(r1)?;
        let pc_offset = sign_extend(instr & 0b0011_1111, 6)?;
        let address = value_in_r1.wrapping_add(pc_offset);
        self.mem_write(value_in_r0, address.into())?;
        Ok(())
    }

    /// TRAP
    ///
    /// Facilitates the interaction with the user, allows to read and write in standard input/ouput.
    /// Also can Halt the VM
    fn trap(&mut self, instr: u16) -> Result<(), VMError> {
        self.set_register(7, self.pc)?;
        let trap_code: TrapCode = (instr & 0b1111_1111).try_into()?;
        match trap_code {
            TrapCode::Getc => self.getc(),
            TrapCode::Out => self.out(),
            TrapCode::Puts => self.puts(),
            TrapCode::IN => self.in_trap(),
            TrapCode::Putsp => self.putsp(),
            TrapCode::Halt => self.halt(),
        }?;
        Ok(())
    }

    /// GETC
    ///
    /// Reads one character from the standard input. It's stored in R0
    fn getc(&mut self) -> Result<(), VMError> {
        let mut buffer: [u8; 1] = [0];
        stdin()
            .read_exact(&mut buffer)
            .map_err(|e| VMError::StandardIO(format!("Cannot read from Standard Input: {}", e)))?;
        self.set_register(0, buffer[0].into())?;
        self.update_flags(0)?;
        Ok(())
    }

    /// OUT
    ///
    /// Write one character from R0 into the standard output.
    fn out(&mut self) -> Result<(), VMError> {
        let char: u8 = (*self.get_register(0)?)
            .try_into()
            .map_err(|_| VMError::InvalidCharacter)?;
        let char: char = char.into();
        print!("{char}");
        stdout()
            .flush()
            .map_err(|e| VMError::StandardIO(format!("Could not flush output: {e}")))?;
        Ok(())
    }

    /// PUTS
    ///
    /// Writes from the address stored in R0 the characters into the standard output.
    /// Each memory position (16 bits) is interpreted as a single character
    fn puts(&mut self) -> Result<(), VMError> {
        let mut address = *self.get_register(0)?;
        let mut char_memory = self.mem_read(address.into())?;
        while char_memory != 0 {
            let char: u8 = char_memory
                .try_into()
                .map_err(|_| VMError::InvalidCharacter)?;
            let char: char = char.into();
            print!("{char}");
            address = address
                .checked_add(1)
                .ok_or(VMError::MemoryIndex(String::from("String too long")))?;
            char_memory = self.mem_read(address.into())?;
        }
        stdout()
            .flush()
            .map_err(|e| VMError::StandardIO(format!("Could not flush output: {e}")))?;
        Ok(())
    }

    /// IN
    ///
    /// Prompt the user to insput a character. It's echoed into the standard output
    fn in_trap(&mut self) -> Result<(), VMError> {
        println!("Enter a character: ");
        let mut buffer: [u8; 1] = [0];
        stdin()
            .read_exact(&mut buffer)
            .map_err(|e| VMError::StandardIO(format!("Cannot read from Standard Input: {}", e)))?;
        self.set_register(0, buffer[0].into())?;

        // write char
        let char: char = buffer[0].into();
        print!("{char}");

        self.update_flags(0)?;
        stdout()
            .flush()
            .map_err(|e| VMError::StandardIO(format!("Could not flush output: {e}")))?;
        Ok(())
    }

    /// PUTSP
    ///
    /// Writes from the address stored in R0 the characters into the standard output.
    /// Each memory position (16 bits) is interpreted as two characters, prints two chars per
    /// position. Conversion from little endian to big endian is made on each position
    fn putsp(&mut self) -> Result<(), VMError> {
        let mut address = *self.get_register(0)?;
        let mut char_memory = self.mem_read(address.into())?;
        while char_memory != 0 {
            // write first char
            let first_char = char_memory & 0b1111_1111;
            let char: u8 = first_char
                .try_into()
                .map_err(|_| VMError::InvalidCharacter)?;
            let char: char = char.into();
            print!("{char}");

            // write second char
            let second_char = char_memory >> 8;
            let char: u8 = second_char
                .try_into()
                .map_err(|_| VMError::InvalidCharacter)?;
            let char: char = char.into();
            print!("{char}");

            address = address
                .checked_add(1)
                .ok_or(VMError::MemoryIndex(String::from("String too long")))?;
            char_memory = self.mem_read(address.into())?;
        }
        stdout()
            .flush()
            .map_err(|e| VMError::StandardIO(format!("Could not flush output: {e}")))?;
        Ok(())
    }

    /// HALT
    ///
    /// Stops the execution from the Virtual Machine
    fn halt(&mut self) -> Result<(), VMError> {
        println!("HALT");
        stdout()
            .flush()
            .map_err(|e| VMError::StandardIO(format!("Could not flush output: {e}")))?;
        self.running = false;
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
        assert_eq!(vm.mem_read(0x3000)?, 0xf3f2);
        assert_eq!(vm.mem_read(0x3001)?, 0xf5f4);
        assert_eq!(vm.mem_read(0x3002)?, 0xf7f6);
        Ok(())
    }

    #[test]
    fn add_to_registers_and_store() -> Result<(), VMError> {
        let instr: u16 = 0b0001_0000_0100_0010; // ADD R0, R1, R2
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
        let instr: u16 = 0b0001_0000_0100_0010; // ADD R0, R1, R2
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
        let instr: u16 = 0b0001_0000_0100_0010; // ADD R0, R1, R2
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
        let instr: u16 = 0b0001_0000_0110_0010; // ADD R0, R1, #2
        let mut vm = VM::new();
        vm.registers[1] = 1;
        vm.add(instr)?;
        assert_eq!(vm.registers[0], 0x3);
        assert_eq!(vm.cond, ConditionFlag::Pos);
        Ok(())
    }

    #[test]
    fn and_register() -> Result<(), VMError> {
        let instr: u16 = 0b0101_0000_0100_0010; // AND R0, R1, R2
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
        let instr: u16 = 0b0001_0000_0110_0111; // AND R0, R1, #7
        let mut vm = VM::new();
        vm.registers[1] = 0xFFFF;
        vm.and(instr)?;
        assert_eq!(vm.registers[0], 0b0111);
        assert_eq!(vm.cond, ConditionFlag::Pos);
        Ok(())
    }

    #[test]
    fn not_register() -> Result<(), VMError> {
        let instr: u16 = 0b1001_0000_0111_1111; // NOT R0, R1
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
        vm.mem_write(0x4242, 0x3042)?;
        vm.ld(instr)?;
        assert_eq!(vm.registers[0], 0x4242);
        Ok(())
    }

    #[test]
    fn load_register_to_register() -> Result<(), VMError> {
        let mut vm = VM::new();
        let instr: u16 = 0b0110_0000_0100_0010; // load in R0, value stored in R1 + 0x02
        vm.registers[1] = 0x3040;
        vm.mem_write(0x4242, 0x3042)?;
        vm.ld(instr)?;
        assert_eq!(vm.registers[0], 0x4242);
        Ok(())
    }

    #[test]
    fn load_indirect() -> Result<(), VMError> {
        let mut vm = VM::new();
        let instr: u16 = 0b1010_0000_0100_0010; // pc_offset is 0x42, will look for address in 0x3042
        vm.mem_write(0x4242, 0x3042)?;
        vm.mem_write(0x5353, 0x4242)?;
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
        assert_eq!(vm.mem_read(0x3042)?, 0x4242);
        Ok(())
    }

    #[test]
    fn store_indirect_value() -> Result<(), VMError> {
        let mut vm = VM::new();
        let instr: u16 = 0b1011_0000_0100_0010; // pc_offset is 0x42, store what is in R0 in [0x3042]
        vm.mem_write(0x5353, 0x3042)?;
        vm.registers[0] = 0x4242;
        vm.sti(instr)?;
        assert_eq!(vm.mem_read(0x5353)?, 0x4242);
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
        assert_eq!(vm.mem_read(0x1236)?, 0x4242);
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
        assert_eq!(vm.mem_read(0x3000)?, 0x5353);
        Ok(())
    }

    #[test]
    fn puts_print_4_chars() -> Result<(), VMError> {
        let mut vm = VM::new();
        vm.mem_write(0x0064, 0x4000)?; // d
        vm.mem_write(0x0065, 0x4001)?; // e
        vm.mem_write(0x0066, 0x4002)?; // f
        vm.mem_write(0x0067, 0x4003)?; // g
        vm.registers[0] = 0x4000;
        // Shouldn't fail
        vm.puts()?;
        Ok(())
    }

    #[test]
    fn putsp_print_8_chars() -> Result<(), VMError> {
        let mut vm = VM::new();
        vm.mem_write(0x6564, 0x4000)?; // d e in little endian
        vm.mem_write(0x6766, 0x4001)?; // f g
        vm.mem_write(0x6968, 0x4002)?; // h i
        vm.mem_write(0x6B6A, 0x4003)?; // j k
        vm.registers[0] = 0x4000;
        // Shouldn't fail
        vm.putsp()?;
        Ok(())
    }

    #[test]
    fn halt_stops_running() -> Result<(), VMError> {
        let mut vm = VM::new();
        vm.halt()?;
        assert!(!vm.running);
        Ok(())
    }

    #[test]
    fn halt_executing_trap() -> Result<(), VMError> {
        let mut vm = VM::new();
        let instr: u16 = 0b1111_0000_0010_0101; // TRAP HALT
        vm.execute(instr)?;
        assert!(!vm.running);
        Ok(())
    }
}
