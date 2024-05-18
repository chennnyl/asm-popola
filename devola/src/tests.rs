use crate::instructions::*;
use crate::parser;
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
    let code: Vec<Instruction> = parser::intermediate::process_labels(vec![
            Instruction::Load       (Register::Accumulator, AddressingMode::Immediate   (0)),
            Instruction::Load       (Register::UtilityB,    AddressingMode::Immediate   (5)),
            Instruction::Load       (Register::UtilityC,    AddressingMode::Immediate   (0)),
        Instruction::_Label(String::from("loop")),
            Instruction::Compare                            (AddressingMode::Register   (Register::UtilityB)),
            Instruction::_LabeledJump(JumpType::Flag(Flag::Zero, true), String::from("end_loop")),
            Instruction::Push       (Register::Accumulator),
            Instruction::Load       (Register::Accumulator, AddressingMode::Register    (Register::UtilityC)),
            Instruction::Add                                (AddressingMode::Register   (Register::UtilityB)),
            Instruction::Load       (Register::UtilityC,    AddressingMode::Register    (Register::Accumulator)),
            Instruction::Pop        (Register::Accumulator),
            Instruction::Increment,
            Instruction::_LabeledJump(JumpType::Unconditional, String::from("loop")),
        Instruction::_Label(String::from("end_loop")),
            Instruction::_Assert(AddressingMode::Register(Register::UtilityC), 25)
    ]).unwrap();

    let devola = Devola::new(code);
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
    let code: Vec<Instruction> = parser::intermediate::process_labels(vec![
            Instruction::_LabeledJump(JumpType::Unconditional, String::from("main")),

        Instruction::_Label(String::from("square")),
            Instruction::Push       (Register::Accumulator),
            Instruction::Push       (Register::UtilityC),
            Instruction::Load       (Register::Accumulator, AddressingMode::Immediate   (0)),
            Instruction::Load       (Register::UtilityC,    AddressingMode::Immediate   (0)),

        Instruction::_Label(String::from("loop")),
            Instruction::Compare                            (AddressingMode::Register   (Register::UtilityB)),
            Instruction::_LabeledJump(JumpType::Flag(Flag::Zero, true), String::from("end_loop")),
            Instruction::Push       (Register::Accumulator),
            Instruction::Load       (Register::Accumulator, AddressingMode::Register    (Register::UtilityC)),
            Instruction::Add                                (AddressingMode::Register   (Register::UtilityB)),
            Instruction::Load       (Register::UtilityC,    AddressingMode::Register    (Register::Accumulator)),
            Instruction::Pop        (Register::Accumulator),
            Instruction::Increment,
            Instruction::_LabeledJump(JumpType::Unconditional, String::from("loop")),
        Instruction::_Label(String::from("end_loop")),
            Instruction::Load       (Register::UtilityB,    AddressingMode::Register    (Register::UtilityC)),
            Instruction::Pop        (Register::UtilityC),
            Instruction::Pop        (Register::Accumulator),
            Instruction::Return,

            Instruction::_Assert(AddressingMode::Immediate(0), 1), // unreachable

        Instruction::_Label(String::from("main")),
            Instruction::Load       (Register::UtilityB,    AddressingMode::Immediate   (13)),
            Instruction::_LabeledCall(String::from("square")),
            Instruction::_Assert(AddressingMode::Register(Register::UtilityB), 13*13),

            Instruction::Load       (Register::UtilityB,    AddressingMode::Immediate   (12)),
            Instruction::_LabeledCall(String::from("square")),
            Instruction::_Assert(AddressingMode::Register(Register::UtilityB), 12*12),

            Instruction::Load       (Register::UtilityB,    AddressingMode::Immediate   (3)),
            Instruction::_LabeledCall(String::from("square")),
            Instruction::_Assert(AddressingMode::Register(Register::UtilityB), 3*3)

    ]).unwrap();

    let devola = Devola::new(code);
    if let Err(_) = devola.run() {
        panic!();
    }
}