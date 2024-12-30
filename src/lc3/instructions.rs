use super::{bytes::sign_extend, vm::*};

impl VM {
    /// ADD
    /// ADD DR, SR1, SR2 or ADD DR, SR1, imm5
    ///
    /// Stores in DR the addition between SR1 and SR2 or SR1 and imm5 sign extended
    /// Bit 5 if it is 1, then is read as a imm5
    pub fn add(&mut self, instr: u16) -> Result<(), VMError> {
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
    /// AND DR, SR1, SR2 or AND DR, SR1, imm5
    ///
    /// Stores in DR the and bitwise between SR1 and SR2 or SR1 and imm5 sign extended
    /// Bit 5 if it is 1, then is read as a imm5
    pub fn and(&mut self, instr: u16) -> Result<(), VMError> {
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
    /// NOT DR, SR1
    ///
    /// Stores in DR the negate bitwise of SR1
    pub fn not(&mut self, instr: u16) -> Result<(), VMError> {
        let r0 = (instr >> 9) & 0b0111;
        let r1 = (instr >> 6) & 0b0111;
        let value_in_r1 = *self.get_register(r1)?;
        self.set_register(r0, !value_in_r1)?;
        self.update_flags(r0)?;
        Ok(())
    }

    /// BR
    /// BRnzp PCoffset9
    ///
    /// In bit 11 is to jump if is negative
    /// In bit 10 is to jump if is zero
    /// In bit 9 is to jump if positive
    ///
    /// Stores in PC the addition of the PC and the sign extended PCoffset9 only if the condition flag
    /// meets any of the bits that are 1
    pub fn br(&mut self, instr: u16) -> Result<(), VMError> {
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

    /// JMP
    /// JMP BaseR
    ///
    /// Stores in PC the addition of the PC and the value stored in the register BaseR
    pub fn jmp(&mut self, instr: u16) -> Result<(), VMError> {
        let r1 = (instr >> 6) & 0b0111;
        let value_in_r1 = *self.get_register(r1)?;
        self.pc = value_in_r1;
        Ok(())
    }

    /// JSR/JSRR
    /// JSR PCoffset11 or JSRR BaseR
    ///
    /// If bit 11 is a 1, then PC is added with the sign extension of PCoffset11
    /// Else, PC is added with the value in the register
    pub fn jsr(&mut self, instr: u16) -> Result<(), VMError> {
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

    /// LEA
    /// LEA DR, LABEL
    ///
    /// Stores in the register DR the value of the label
    pub fn lea(&mut self, instr: u16) -> Result<(), VMError> {
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

    /// LD
    /// LD DR, PCoffset9
    ///
    /// Loads the value in memory from the address that is obtained sign extending PCoffset9 plus the PC
    /// into the register DR
    pub fn ld(&mut self, instr: u16) -> Result<(), VMError> {
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

    /// LDR
    /// LDR DR, BaseR, offset6
    ///
    /// Stores in DR the value from memory that is obtained sign extending offset6 plus
    /// the value in the register BaseR
    pub fn ldr(&mut self, instr: u16) -> Result<(), VMError> {
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

    /// LDI
    /// LDI DR, PCoffset9
    ///
    /// Stores in the register DR the value in memory where the address is stored in the position in memory of
    /// the PC plus the sign extension of PCoffset9
    pub fn ldi(&mut self, instr: u16) -> Result<(), VMError> {
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

    /// ST
    /// ST SR, PCoffset9
    ///
    /// Stores in the address of the PC plus the sign extension of PCoffset9 the value in
    /// the register SR
    pub fn st(&mut self, instr: u16) -> Result<(), VMError> {
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

    /// STI
    /// STI SR, PCoffset9
    ///
    /// Stores in the address that is stored in memory in the position of the PC plus the sign extension of PCoffset9,
    /// the value in the register SR
    pub fn sti(&mut self, instr: u16) -> Result<(), VMError> {
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

    /// STR
    /// STR SR, BaseR, offset6
    ///
    /// Stores in the address of the BaseR plus the sign extension of PCoffset6 the value in
    /// the register SR
    pub fn str(&mut self, instr: u16) -> Result<(), VMError> {
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
}
