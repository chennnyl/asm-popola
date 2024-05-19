use std::collections::HashMap;
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
    pc: usize,
    debug: bool,
    call_stack: Vec<String>,
    symbol_table: Option<HashMap<usize, String>>
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
    pub fn new(code: Vec<Instruction>, symbol_table: Option<HashMap<usize, String>>) -> Self {
        let mut out = Self {
            memory: DevolaMemory::new(),
            code,
            pc: 0,
            debug: false,
            call_stack: Vec::new(),
            symbol_table
        };
        let (msb, lsb) = break_u16(INITIAL_STACK_POINTER);
        out.memory[STACK_POINTER_MSB] = msb;
        out.memory[STACK_POINTER_LSB] = lsb;

        out
    }

    pub fn enable_debug(&mut self) {
        self.debug = true;
    }
    pub fn disable_debug(&mut self) {
        self.debug = false;
    }

    pub fn run(mut self) -> Result<(), DevolaError> {
        let mut modified_addresses: HashMap<u16, u8> = HashMap::new();

        loop {
            match self.code.get(self.pc) {
                Some(instruction) => {
                    let debug_inst = instruction.clone();
                    if let Err(error) = self.execute_instruction(instruction.clone()) {
                        if self.debug {
                            eprintln!("An error of type {:?} occurred at PC {}", error, self.pc);
                        }
                        return Err(error);
                    }
                    if self.debug {
                        match debug_inst {
                            Instruction::Store(_, mode) => {
                                let lvalue = match mode {
                                    AddressingMode::Indirect(address) => address,
                                    AddressingMode::Index => self.memory.get_index(),
                                    _ => 0
                                };
                                let rvalue = self.resolve_rvalue(mode.clone());

                                modified_addresses.insert(lvalue, rvalue);
                            }
                            Instruction::Call(CallType::Local(loc)) => {
                                let symbol = match &self.symbol_table {
                                    Some(table) => table.get(&loc).unwrap_or(&String::from("unknown")).clone(),
                                    None => loc.to_string()
                                };

                                println!("Call {}", symbol);
                                self.call_stack.push(symbol);
                            }
                            Instruction::Return => {
                                println!("{} returned {}", self.call_stack.pop().unwrap_or(String::from("unknown")), self.memory[Register::UtilityB]);
                            },
                            _ => {}
                        };
                    }
                    self.pc += 1;
                }
                None => { break }
            }
        }

        if self.debug {
            println!("{:?}", modified_addresses);
            println!("Registers:\nA: 0x{:02x} B: 0x{:02x} C: 0x{:02x} XY: 0x{:04x}",
                     self.memory[Register::Accumulator], self.memory[Register::UtilityB],
                     self.memory[Register::UtilityC], self.memory.get_index()
            );
            println!("Flags:\nC: {:?} P: {:?} S: {:?} Z: {:?}",
                     self.memory.flag(Flag::Carry), self.memory.flag(Flag::Parity),
                     self.memory.flag(Flag::Sign), self.memory.flag(Flag::Zero)
            );
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


#[cfg(test)]
mod tests {
    use std::path::Path;
    use crate::instructions::*;
    use crate::parser;
    use crate::vm::*;

    /// Tests the equivalent of:
    /// ```asm
    /// lda 10   ; a = 10
    /// sta 5    ; mem[5] = 10
    /// ldx a    ; x = 10
    /// ldx 0xF0 ; x = 0xF0
    /// ldy 0x00 ; y = 0x00
    /// stx xy   ; mem[0xF000] = 0xF0
    /// ```
    #[test]
    fn test_load_store() {
        let code: Vec<Instruction> = vec![
            Instruction::Load(Register::Accumulator, AddressingMode::Immediate(10)),
            Instruction::_Assert(
                AddressingMode::Register(Register::Accumulator), 10
            ),
            Instruction::Store(Register::Accumulator, AddressingMode::Indirect(5)),
            Instruction::_Assert(
                AddressingMode::Indirect(5), 10
            ),
            Instruction::Load(Register::IndexX, AddressingMode::Register(Register::Accumulator)),
            Instruction::_Assert(
                AddressingMode::Register(Register::IndexX), 10
            ),
            Instruction::Load(Register::IndexX, AddressingMode::Immediate(0xF0)),
            Instruction::_Assert(
                AddressingMode::Register(Register::IndexX), 0xF0
            ),
            Instruction::Load(Register::IndexY, AddressingMode::Immediate(0x00)),
            Instruction::_Assert(
                AddressingMode::Register(Register::IndexY), 0x00
            ),
            Instruction::Store(Register::IndexX, AddressingMode::Index),
            Instruction::_Assert(
                AddressingMode::Indirect(0xF000), 0xF0
            )
        ];

        let devola = Devola::new(code, None);
        if let Err(_) = devola.run() {
            panic!();
        }
    }

    #[test]
    /// Tests the equivalent of
    /// ```asm
    ///     lda 0       ; i = 0
    ///     ldb 5       ; n = 5
    ///     ldc 0       ; square = 0
    /// loop:           ; while true
    ///     cmp b       ; if i == n break
    ///     jz end_loop
    ///     push a      ; square += n
    ///     lda c
    ///     add b
    ///     ldc a
    ///     pop a
    ///     inc         ; i++
    ///     jmp loop
    /// end_loop:
    fn test_square() {
        let (code, _) = parser::intermediate::process_labels(vec![
            Instruction::Load(Register::Accumulator, AddressingMode::Immediate(0)),
            Instruction::Load(Register::UtilityB, AddressingMode::Immediate(5)),
            Instruction::Load(Register::UtilityC, AddressingMode::Immediate(0)),
            Instruction::_Label(String::from("loop")),
            Instruction::Compare(AddressingMode::Register(Register::UtilityB)),
            Instruction::_LabeledJump(JumpType::Flag(Flag::Zero, true), String::from("end_loop")),
            Instruction::Push(Register::Accumulator),
            Instruction::Load(Register::Accumulator, AddressingMode::Register(Register::UtilityC)),
            Instruction::Add(AddressingMode::Register(Register::UtilityB)),
            Instruction::Load(Register::UtilityC, AddressingMode::Register(Register::Accumulator)),
            Instruction::Pop(Register::Accumulator),
            Instruction::Increment,
            Instruction::_LabeledJump(JumpType::Unconditional, String::from("loop")),
            Instruction::_Label(String::from("end_loop")),
            Instruction::_Assert(AddressingMode::Register(Register::UtilityC), 25)
        ]).unwrap();

        let devola = Devola::new(code, None);
        if let Err(_) = devola.run() {
            panic!();
        }
    }

    #[test]
    /// Tests the equivalent of
    /// ```asm
    ///     jmp main
    ///
    /// ; place number to square in b, square will be returned there
    /// square:
    ///     push a
    ///     push c
    ///     lda 0       ; i = 0
    ///     ldc 0       ; square = 0
    /// loop:           ; while true
    ///     cmp b       ; if i == n break
    ///     jz end_loop
    ///     push a      ; square += n
    ///     lda c
    ///     add b
    ///     ldc a
    ///     pop a
    ///     inc         ; i++
    ///     jmp loop
    /// end_loop:
    ///     ldb c
    ///     pop c
    ///     pop a
    ///     ret
    ///
    /// main:
    ///     ldb 13
    ///     call square
    ///
    ///     ldb 12
    ///     call square
    ///
    ///     ldb 3
    ///     call square
    fn test_subroutine_square() {
        let (code, _) = parser::intermediate::process_labels(vec![
            Instruction::_LabeledJump(JumpType::Unconditional, String::from("main")),
            Instruction::_Label(String::from("square")),
            Instruction::Push(Register::Accumulator),
            Instruction::Push(Register::UtilityC),
            Instruction::Load(Register::Accumulator, AddressingMode::Immediate(0)),
            Instruction::Load(Register::UtilityC, AddressingMode::Immediate(0)),
            Instruction::_Label(String::from("loop")),
            Instruction::Compare(AddressingMode::Register(Register::UtilityB)),
            Instruction::_LabeledJump(JumpType::Flag(Flag::Zero, true), String::from("end_loop")),
            Instruction::Push(Register::Accumulator),
            Instruction::Load(Register::Accumulator, AddressingMode::Register(Register::UtilityC)),
            Instruction::Add(AddressingMode::Register(Register::UtilityB)),
            Instruction::Load(Register::UtilityC, AddressingMode::Register(Register::Accumulator)),
            Instruction::Pop(Register::Accumulator),
            Instruction::Increment,
            Instruction::_LabeledJump(JumpType::Unconditional, String::from("loop")),
            Instruction::_Label(String::from("end_loop")),
            Instruction::Load(Register::UtilityB, AddressingMode::Register(Register::UtilityC)),
            Instruction::Pop(Register::UtilityC),
            Instruction::Pop(Register::Accumulator),
            Instruction::Return,
            Instruction::_Assert(AddressingMode::Immediate(0), 1), // unreachable

            Instruction::_Label(String::from("main")),
            Instruction::Load(Register::UtilityB, AddressingMode::Immediate(13)),
            Instruction::_LabeledCall(String::from("square")),
            Instruction::_Assert(AddressingMode::Register(Register::UtilityB), 13 * 13),
            Instruction::Load(Register::UtilityB, AddressingMode::Immediate(12)),
            Instruction::_LabeledCall(String::from("square")),
            Instruction::_Assert(AddressingMode::Register(Register::UtilityB), 12 * 12),
            Instruction::Load(Register::UtilityB, AddressingMode::Immediate(3)),
            Instruction::_LabeledCall(String::from("square")),
            Instruction::_Assert(AddressingMode::Register(Register::UtilityB), 3 * 3)
        ]).unwrap();

        let devola = Devola::new(code, None);
        if let Err(_) = devola.run() {
            panic!();
        }
    }

    fn execute_file(path: &str) {
        let file = Path::new(path);
        let code = crate::util::read_from_file(file);

        let (code, symbols) = parser::text::compile(code).unwrap();

        let mut devola = Devola::new(code, Some(symbols));
        devola.enable_debug();

        if let Err(_) = devola.run() {
            panic!();
        }
    }

    #[test]
    fn test_compile_run_from_source_squares() {
        execute_file("sample/square.pop")
    }

    #[test]
    fn test_compile_run_from_source_squares_subroutines() {
        execute_file("sample/square_subroutines.pop");
    }

    #[test]
    fn test_compile_run_from_source_rw() {
        execute_file("sample/read_write_memory.pop");
    }
}

// TODO: add adxy/sbxy for word arithmetic
// TODO: allow offsetting index register