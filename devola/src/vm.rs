use std::ops::{Index, IndexMut};
use crate::instructions::*;

const MEMORY_SIZE: usize = (u16::MAX as usize)+1;
/// There are 16 bytes of memory-mapped I/O (MMIO). They are labeled as (relative to the base MMIO address):
/// - `0`: MSB of the stack pointer
/// - `1`: LSB of the stack pointer
/// - `2`-`F`: Reserved
const MMIO: u16             = 0x0FF0;
const STACK_POINTER_MSB: u16    = MMIO+0x1;
const STACK_POINTER_LSB: u16    = MMIO+0x2;
// const VRAM: u16 = 0xF000;

/// The stack begins at 0x0F00 and grows down
const INITIAL_STACK_POINTER: u16 = 0x0F00;

struct DevolaMemory {
    memory: [u8; MEMORY_SIZE],
    flags: u8,
    registers: [u8; 5]
}

impl Index<u16> for DevolaMemory {
    type Output = u8;
    fn index(&self, index: u16) -> &Self::Output {
        &self.memory[index as usize]
    }
}

impl IndexMut<u16> for DevolaMemory {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.memory[index as usize]
    }
}

impl Index<Register> for DevolaMemory {
    type Output = u8;
    fn index(&self, index: Register) -> &Self::Output {
        &self.registers[match index {
            Register::Accumulator => 0,
            Register::IndexX => 1,
            Register::IndexY => 2,
            Register::UtilityB => 3,
            Register::UtilityC => 4
        }]
    }
}

impl IndexMut<Register> for DevolaMemory {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        &mut self.registers[match index {
            Register::Accumulator => 0,
            Register::IndexX => 1,
            Register::IndexY => 2,
            Register::UtilityB => 3,
            Register::UtilityC => 4
        }]
    }
}

impl DevolaMemory {
    pub fn new() -> Self {
        Self {
            memory: [0; MEMORY_SIZE],
            flags: 0,
            registers: [0; 5]
        }
    }
    /// Checks if the specified flag has been set.
    /// Flags are laid out as `0b0000SPZC`.
    pub fn flag(&self, flag: Flag) -> bool {
        (self.flags >> match flag {
            Flag::Carry => 0,
            Flag::Zero => 1,
            Flag::Parity => 2,
            Flag::Sign => 3
        } & 1) == 1
    }

    fn flag_mask(flag: Flag) -> u8 {
        match flag {
            Flag::Carry     => 0b1111_1110,
            Flag::Zero      => 0b1111_1101,
            Flag::Parity    => 0b1111_1011,
            Flag::Sign      => 0b1111_0111
        }
    }
    pub fn clear_flag(&mut self, flag: Flag) {
        self.flags &= Self::flag_mask(flag);
    }
    pub fn set_flag(&mut self, flag:Flag) {
        self.flags |= !Self::flag_mask(flag);
    }

    pub fn get_index(&self) -> u16 {
        ((self[Register::IndexX] as u16) << 8) | self[Register::IndexY] as u16
    }
}

pub struct Devola {
    memory: DevolaMemory,
    code: Vec<Instruction>,
    pc: usize
}
#[derive(Copy, Clone, Debug)]
pub enum DevolaError {
    InvalidArgument, Unimplemented
}

fn build_u16(msb: u8, lsb: u8) -> u16 {
    ((msb as u16) << 8) | lsb as u16
}
fn break_u16(word: u16) -> (u8, u8) {
    ((word >> 8) as u8, (word & 0x00FF) as u8)
}

impl Devola {
    pub fn new(code: Vec<Instruction>) -> Self {
        let mut out = Self {
            memory: DevolaMemory::new(),
            code,
            pc: 0
        };
        let (msb, lsb) = break_u16(INITIAL_STACK_POINTER);
        out.memory[STACK_POINTER_MSB] = msb;
        out.memory[STACK_POINTER_LSB] = lsb;

        out
    }

    pub fn run(mut self) -> Result<(), DevolaError> {
        loop {
            match self.code.get(self.pc) {
                Some(instruction) => {
                    if let Err(error) = self.execute_instruction(instruction.clone()) {
                        eprintln!("An error of type {:?} occurred at PC {}", error, self.pc);
                        return Err(error);
                    }
                    self.pc += 1;
                }
                None => { break }
            }
        }
        Ok(())
    }

    fn push(&mut self, value: u8) {
        let new_stack_pointer = self.get_stack_pointer()-1;
        let (msb, lsb) = break_u16(new_stack_pointer);
        self.memory[new_stack_pointer] = value;
        self.memory[STACK_POINTER_MSB] = msb;
        self.memory[STACK_POINTER_LSB] = lsb;
    }
    fn pop(&mut self) -> u8 {
        let new_stack_pointer = self.get_stack_pointer()+1;
        let (msb, lsb) = break_u16(new_stack_pointer);
        self.memory[STACK_POINTER_MSB] = msb;
        self.memory[STACK_POINTER_LSB] = lsb;

        self.memory[new_stack_pointer-1]
    }

    fn resolve_rvalue(&self, addressing_mode: AddressingMode) -> u8 {
        match addressing_mode {
            AddressingMode::Register(register) => self.memory[register],
            AddressingMode::Immediate(value) => value,
            AddressingMode::Indirect(source) => self.memory[source],
            AddressingMode::Index => self.memory[self.memory.get_index()]
        }
    }

    fn get_stack_pointer(&self) -> u16 {
        build_u16(self.memory[STACK_POINTER_MSB], self.memory[STACK_POINTER_LSB])
    }

    fn execute_instruction(&mut self, instruction: Instruction) -> Result<(), DevolaError> {
        match instruction {
            Instruction::Load(dest_register, addressing_mode) => {
                let value = self.resolve_rvalue(addressing_mode);
                self.memory[dest_register] = value;
                Ok(())
            }
            Instruction::Store(register, addressing_mode) => {
                let dest_byte = match addressing_mode {
                    AddressingMode::Register(_) | AddressingMode::Immediate(_) => { return Err(DevolaError::InvalidArgument) }
                    AddressingMode::Indirect(pointer) => pointer,
                    AddressingMode::Index => self.memory.get_index()
                };
                self.memory[dest_byte] = self.memory[register];
                Ok(())
            }
            Instruction::Increment => {
                self.memory.clear_flag(Flag::Zero);
                self.memory.clear_flag(Flag::Sign);
                self.memory.clear_flag(Flag::Parity);

                if self.memory[Register::Accumulator] == 0xFF {
                    self.memory[Register::Accumulator] = 0;
                    self.memory.set_flag(Flag::Zero);
                } else {
                    self.memory[Register::Accumulator] += 1;
                }
                let result = self.memory[Register::Accumulator];

                if result % 2 == 1 {
                    self.memory.set_flag(Flag::Parity);
                }
                if result & 0x80 == 0x80 {
                    self.memory.set_flag(Flag::Sign);
                }

                Ok(())
            }
            Instruction::Decrement => {
                self.memory.clear_flag(Flag::Zero);
                self.memory.clear_flag(Flag::Sign);
                self.memory.clear_flag(Flag::Parity);

                if self.memory[Register::Accumulator] == 0x00 {
                    self.memory[Register::Accumulator] = 0xFF;
                    self.memory.set_flag(Flag::Zero);
                } else {
                    self.memory[Register::Accumulator] -= 1;
                }
                let result = self.memory[Register::Accumulator];

                if result % 2 == 1 {
                    self.memory.set_flag(Flag::Parity);
                }
                if result & 0x80 == 0x80 {
                    self.memory.set_flag(Flag::Sign);
                }

                Ok(())
            }
            Instruction::Add(addressing_mode) => {
                self.memory.clear_flag(Flag::Zero);
                self.memory.clear_flag(Flag::Sign);
                self.memory.clear_flag(Flag::Parity);
                self.memory.clear_flag(Flag::Carry);

                let addand = self.resolve_rvalue(addressing_mode);
                let accumulator = self.memory[Register::Accumulator];

                let (result, carry) = accumulator.overflowing_add(addand);
                self.memory[Register::Accumulator] = result;

                if result == 0 {
                    self.memory.set_flag(Flag::Zero);
                }
                if result & 0x80 == 0x80 {
                    self.memory.set_flag(Flag::Sign);
                }
                if result % 2 == 1 {
                    self.memory.set_flag(Flag::Parity);
                }
                if carry {
                    self.memory.set_flag(Flag::Carry);
                }

                Ok(())
            }
            Instruction::Subtract(addressing_mode) => {
                self.memory.clear_flag(Flag::Zero);
                self.memory.clear_flag(Flag::Sign);
                self.memory.clear_flag(Flag::Parity);
                self.memory.clear_flag(Flag::Carry);

                let addand = self.resolve_rvalue(addressing_mode);
                let accumulator = self.memory[Register::Accumulator];

                let (result, carry) = accumulator.overflowing_sub(addand);
                self.memory[Register::Accumulator] = result;

                if result == 0 {
                    self.memory.set_flag(Flag::Zero);
                }
                if result & 0x80 == 0x80 {
                    self.memory.set_flag(Flag::Sign);
                }
                if result % 2 == 1 {
                    self.memory.set_flag(Flag::Parity);
                }
                if carry {
                    self.memory.set_flag(Flag::Carry);
                }

                Ok(())
            }
            Instruction::Compare(addressing_mode) => {
                self.memory.clear_flag(Flag::Zero);
                self.memory.clear_flag(Flag::Sign);
                self.memory.clear_flag(Flag::Parity);
                self.memory.clear_flag(Flag::Carry);

                let comparator = self.resolve_rvalue(addressing_mode);
                let accumulator = self.memory[Register::Accumulator];

                if comparator == accumulator {
                    self.memory.set_flag(Flag::Zero);
                }
                if (comparator & 0x80) == (accumulator & 0x80) {
                    self.memory.set_flag(Flag::Sign);
                }
                if (comparator % 2) == (accumulator % 2) {
                    self.memory.set_flag(Flag::Parity);
                }
                if accumulator < comparator {
                    self.memory.set_flag(Flag::Carry);
                }

                Ok(())
            }
            Instruction::Jump(jump_type, destination) => {
                match jump_type {
                    JumpType::Unconditional => {
                        self.pc = destination;
                    }
                    JumpType::Flag(flag, set) => {
                        if (self.memory.flag(flag) && set) || (!self.memory.flag(flag) && !set) {
                            self.pc = destination;
                        }
                    }
                };
                Ok(())
            }
            Instruction::Call(call_type) => {
                // TODO: figure out what to do if pc is outside u16 bounds
                match call_type {
                    CallType::Local(dest) => {
                        let (msb, lsb) = break_u16(self.pc as u16);
                        self.push(msb);
                        self.push(lsb);
                        self.pc = dest as usize;
                        Ok(())
                    }
                    CallType::Library(_) => Err(DevolaError::Unimplemented)
                }
            }
            Instruction::Return => {
                let lsb = self.pop();
                let msb = self.pop();
                self.pc = build_u16(msb, lsb) as usize;
                Ok(())
            }
            Instruction::Push(register) => {
                self.push(self.memory[register]);
                Ok(())
            }
            Instruction::Pop(register) => {
                self.memory[register] = self.pop();
                Ok(())
            }
            Instruction::Nop | Instruction::_Label(_) | Instruction::_LabeledJump(_, _) | Instruction::_LabeledCall(_) => Ok(()),
            Instruction::_Assert(addressing_mode, value) => {
                assert_eq!(self.resolve_rvalue(addressing_mode), value);
                Ok(())
            }
        }
    }
}