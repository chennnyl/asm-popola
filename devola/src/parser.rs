use crate::instructions::*;
use std::collections::HashMap;

// TODO: add parsing of actual code rather than like. metaparsing

pub fn process_labels(code: Vec<Instruction>) -> Result<Vec<Instruction>, Vec<(&'static str, usize)>> {
    let jump_table: HashMap<&'static str, usize> = code.iter()
        .enumerate()
        .filter_map(|(pc, instruction)| {
            match instruction {
                &Instruction::_Label(label) => Some((label, pc)),
                _ => None
            }
        })
        .collect();

    let mut missing_labels: Vec<(&'static str, usize)> = Vec::new();

    let filtered = code.iter()
        .enumerate()
        .filter_map(|(line, instruction)| {
            match instruction {
                &Instruction::_LabeledJump(jump_type, label) => {
                    if let Some(pc) = jump_table.get(&label) {
                        Some(Instruction::Jump(jump_type, *pc))
                    } else {
                        missing_labels.push((label, line));
                        None
                    }
                },
                &Instruction::_LabeledCall(label) => {
                    if let Some(pc) = jump_table.get(&label) {
                        Some(Instruction::Call(CallType::Local(*pc)))
                    } else {
                        missing_labels.push((label, line));
                        None
                    }
                },
                &Instruction::_Label(_) => Some(Instruction::Nop),
                _ => Some(instruction.clone())
            }
        })
        .collect();

    if missing_labels.len() > 0 {
        Err(missing_labels)
    } else {
        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::*;

    #[test]
    fn test_processing() {
        let code: Vec<Instruction> = vec![
            Instruction::Load(Register::Accumulator, AddressingMode::Immediate(10)),
            Instruction::_LabeledJump(JumpType::Unconditional, "label"),
            Instruction::_Assert(AddressingMode::Immediate(1), 0),
            Instruction::_Label("label"),
            Instruction::_Assert(AddressingMode::Register(Register::Accumulator), 10),
        ];
        let code = match process_labels(code) {
            Ok(processed) => processed,
            Err(missing_labels) => {
                eprintln!("Encountered the following missing labels:");
                for (label, line) in missing_labels {
                    eprintln!("{} (line {})", label, line+1);
                }
                panic!();
            }
        };

        let devola = Devola::new(code);
        if let Err(_) = devola.run() {
            panic!();
        }
    }

    #[test]
    fn test_missing_labels() {
        let code: Vec<Instruction> = vec![
            Instruction::_LabeledJump(JumpType::Unconditional, "label"),
            Instruction::_LabeledJump(JumpType::Unconditional, "label2"),
            Instruction::_Label("label2")
        ];
        match process_labels(code) {
            Ok(_) => {
                panic!("Did not catch missing labels");
            },
            Err(missing_labels) => {
                assert_eq!(missing_labels.len(), 1);
            }
        };
    }
}