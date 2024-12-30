use std::fs;

use super::{bytes::concatenate_bytes, vm::VMError};

const MEMORY_MAX: usize = 1 << 16;

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

    pub fn mem_read(&self, index: usize) -> Result<u16, VMError> {
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
}
