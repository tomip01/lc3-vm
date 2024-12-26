use std::fs;

const MEMORY_MAX: usize = 1 << 16;
pub struct VM {
    r0: u16,
    r1: u16,
    r2: u16,
    r3: u16,
    r4: u16,
    r5: u16,
    r6: u16,
    r7: u16,
    pc: u16,
    cond: ConditionFlag,
    running: bool,
    memory: [u16; MEMORY_MAX],
}

#[derive(Debug)]
pub enum VMError {
    ReadingFile(String),
    ConcatenatingBytes(String),
    Adding,
}

enum Opcode {
    BR,   /* branch */
    Add,  /* add  */
    LD,   /* load */
    ST,   /* store */
    Jsr,  /* jump register */
    And,  /* bitwise and */
    Ldr,  /* load register */
    Str,  /* store register */
    Rti,  /* unused */
    Not,  /* bitwise not */
    Ldi,  /* load indirect */
    Sti,  /* store indirect */
    Jmp,  /* jump */
    Res,  /* reserved (unused) */
    Lea,  /* load effective address */
    Trap, /* execute trap */
}

enum ConditionFlag {
    Pos, // = 1 << 0, /* P */
    Zro, // = 1 << 1, /* Z */
    Neg, // = 1 << 2, /* N */
}

impl VM {
    pub fn new() -> VM {
        VM {
            r0: 0,
            r1: 0,
            r2: 0,
            r3: 0,
            r4: 0,
            r5: 0,
            r6: 0,
            r7: 0,
            pc: 0x3000,
            cond: ConditionFlag::Zro,
            running: false,
            memory: [0; MEMORY_MAX],
        }
    }

    pub fn read_image(&mut self, image_path: &str) -> Result<(), VMError> {
        let content = &fs::read(image_path)
            .map_err(|_| VMError::ReadingFile(String::from("Error on reading from file path")))?;
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
            let index = i.checked_add(origin).ok_or(VMError::Adding)?;
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
        if bytes.len() != 2 {
            return Err(VMError::ConcatenatingBytes(String::from(
                "Image file is not made from words of 16 bits",
            )));
        }
        let mut res: u16 = (*bytes
            .first()
            .ok_or(VMError::ConcatenatingBytes(String::from(
                "Non existing first bytes",
            )))?)
        .into();
        res <<= 8;
        let second_byte: u16 = (*bytes
            .get(1)
            .ok_or(VMError::ConcatenatingBytes(String::from(
                "Non existing second bytes",
            )))?)
        .into();
        res |= second_byte;
        Ok(res)
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
}
