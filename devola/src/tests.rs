use crate::instructions::*;
use crate::vm::*;

#[cfg(test)]
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

    let devola = Devola::new(code);
    if let Err(_) = devola.run() {
        panic!();
    }
}