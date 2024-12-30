use std::{fs, io::Read};

use super::{bytes::concatenate_bytes, vm::VMError};

const MEMORY_MAX: usize = 1 << 16;
// Keyboard status register
const MR_KBSR: usize = 0xFE00;
// Keyboard data register
const MR_KBDR: usize = 0xFE02;

pub struct Memory {
    memory: [u16; MEMORY_MAX],
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            memory: [0; MEMORY_MAX],
        }
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
            self.mem_write(*word, index)?;
        }

        Ok(())
    }

    pub fn mem_read(&mut self, index: usize) -> Result<u16, VMError> {
        if index == MR_KBSR {
            self.check_key()?;
        }
        let value = self
            .memory
            .get(index)
            .ok_or(VMError::MemoryIndex(String::from(
                "Invalid out of bounds when reading from memory",
            )))?;
        Ok(*value)
    }

    pub fn mem_write(&mut self, value: u16, index: usize) -> Result<(), VMError> {
        let cell = self
            .memory
            .get_mut(index)
            .ok_or(VMError::MemoryIndex(String::from(
                "Index out of bound when writing memory",
            )))?;
        *cell = value;
        Ok(())
    }

    fn check_key(&mut self) -> Result<(), VMError> {
        let mut buffer: [u8; 1] = [0];
        std::io::stdin()
            .read_exact(&mut buffer)
            .map_err(|e| VMError::StandardIO(format!("Cannot read from Standard Input: {}", e)))?;
        if buffer[0] != 0 {
            self.mem_write(1 << 15, MR_KBSR)?;
            self.mem_write(buffer[0].into(), MR_KBDR)?;
        } else {
            self.mem_write(0, MR_KBSR)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_and_write() -> Result<(), VMError> {
        let mut mem = Memory::new();
        mem.mem_write(0x4242, 0x2424)?;
        assert_eq!(0x4242, mem.mem_read(0x2424)?);
        Ok(())
    }

    #[test]
    fn correct_image_read() -> Result<(), VMError> {
        let mut mem = Memory::new();
        // file containing 0x00 0x30 0xf2 0xf3 0xf4 0xf5 0xf6 0xf7
        mem.read_image("images/test-image-load-big-endian")?;
        assert_eq!(mem.mem_read(0x3000)?, 0xf3f2);
        assert_eq!(mem.mem_read(0x3001)?, 0xf5f4);
        assert_eq!(mem.mem_read(0x3002)?, 0xf7f6);
        Ok(())
    }
}
