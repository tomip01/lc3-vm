# LC3-VM

An implementation of the LC3 (Little Computer 3) Virtual Machine in Rust

A VM is a program that acts like a computer. It simulates a CPU along with a few other hardware components, allowing it to perform arithmetic, read and write to memory, and interact with I/O devices, just like a physical computer. Most importantly, it can understand a machine language which you can use to program it.

## Reference

This was made by following this guide: https://www.jmeiners.com/lc3-vm/

## Installation

1. Clone the repository
```bash
git clone https://github.com/tomip01/lc3-vm.git
```
2. Change to the directory:
```bash
cd lc3-vm
```
3. Build project:
```bash
make build
```

## Usage
**Run object file**
```bash
make run FILEPATH=<path/to/file>
```
