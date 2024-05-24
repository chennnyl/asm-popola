pub mod text {
    use crate::instructions::*;
    use regex::{RegexBuilder, Regex};
    use lazy_static::lazy_static;

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum ParseErrorType {
        InvalidRegister, InvalidFlag,
        InvalidNumericLiteral, InvalidInstruction, InvalidLabel
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct ParseError {
        error_type: ParseErrorType,
        location: usize,
        info: Option<String>
    }
    type ParseResult = Result<(Vec<Instruction>, super::intermediate::SymbolTable), Vec<ParseError>>;

    impl TryFrom<char> for Register {
        type Error = ParseError;
        fn try_from(value: char) -> Result<Self, Self::Error> {
            match value {
                'a' | 'A' => Ok(Self::Accumulator),
                'b' | 'B' => Ok(Self::UtilityB),
                'c' | 'C' => Ok(Self::UtilityC),
                'x' | 'X' => Ok(Self::IndexX),
                'y' | 'Y' => Ok(Self::IndexY),
                _ => Err(ParseError {
                    error_type: ParseErrorType::InvalidRegister,
                    location: 0,
                    info: Some(value.to_string())
                })
            }
        }
    }
    impl TryFrom<char> for Flag {
        type Error = ParseError;
        fn try_from(value: char) -> Result<Self, Self::Error> {
            match value {
                'c' | 'C' => Ok(Self::Carry),
                'p' | 'P' => Ok(Self::Parity),
                's' | 'S' => Ok(Self::Sign),
                'z' | 'Z' => Ok(Self::Zero),
                _ => Err(ParseError {
                    error_type: ParseErrorType::InvalidFlag,
                    location: 0,
                    info: Some(value.to_string())
                })
            }
        }
    }

    lazy_static! {
        static ref COMMENTS: Regex = Regex::new(r";.*$").unwrap();
        static ref BLANK_LINES: Regex = Regex::new(r"^\s*$").unwrap();
        static ref DUPLICATE_SPACE: Regex = Regex::new(r"\s{2,}").unwrap();
        static ref LEADING_SPACE: Regex = Regex::new(r"^\s+").unwrap();
        static ref TRAILING_SPACE: Regex = Regex::new(r"\s+$").unwrap();

        static ref ONLY_INDIRECT: &'static str = r"(?<source>#[0-9a-f]+[bh]?|XY)";
        static ref ANY_SOURCE: &'static str = r"(?<source>[abcxy]|#?[0-9a-f]+[bh]?|XY)";

        static ref INST_LOAD: Regex = RegexBuilder::new((String::from(r"ld(?<target>[abcxy]) ") + *ANY_SOURCE).as_str())
            .case_insensitive(true)
            .build()
            .unwrap();
        static ref INST_STORE: Regex = RegexBuilder::new((String::from(r"st(?<target>[abcxy]) ") + *ONLY_INDIRECT).as_str())
            .case_insensitive(true)
            .build()
            .unwrap();
        static ref INST_INC: Regex = RegexBuilder::new(r"inc")
            .case_insensitive(true)
            .build()
            .unwrap();
        static ref INST_DEC: Regex = RegexBuilder::new(r"dec")
            .case_insensitive(true)
            .build()
            .unwrap();
        static ref INST_ADD: Regex = RegexBuilder::new((String::from(r"add ") + *ANY_SOURCE).as_str())
            .case_insensitive(true)
            .build()
            .unwrap();
        static ref INST_SUB: Regex = RegexBuilder::new((String::from(r"sub ") + *ANY_SOURCE).as_str())
            .case_insensitive(true)
            .build()
            .unwrap();
        static ref INST_CMP: Regex = RegexBuilder::new((String::from(r"cmp ") + *ANY_SOURCE).as_str())
            .case_insensitive(true)
            .build()
            .unwrap();
        static ref INST_JUMP: Regex = RegexBuilder::new(r"jmp (?<label>[a-z]\w*)")
            .case_insensitive(true)
            .build()
            .unwrap();
        static ref INST_CONDITIONAL_JUMP: Regex = RegexBuilder::new(r"j(?<condition>n?)(?<flag>[czsp]) (?<label>[a-z]\w*)")
            .case_insensitive(true)
            .build()
            .unwrap();
        static ref INST_CALL: Regex = RegexBuilder::new(r"call (?<label>[a-z]\w*)")
            .case_insensitive(true)
            .build()
            .unwrap();
        static ref INST_RETURN: Regex = RegexBuilder::new(r"ret")
            .case_insensitive(true)
            .build()
            .unwrap();
        static ref INST_NOP: Regex = RegexBuilder::new(r"nop")
            .case_insensitive(true)
            .build()
            .unwrap();
        static ref INST_PUSH: Regex = RegexBuilder::new(r"push (?<source>[abcxy])")
            .case_insensitive(true)
            .build()
            .unwrap();
        static ref INST_POP: Regex = RegexBuilder::new(r"pop (?<source>[abcxy])")
            .case_insensitive(true)
            .build()
            .unwrap();
        static ref INST_LABEL: Regex = Regex::new(r"(?<label>[a-z]\w*):").unwrap();
    }

    fn extract_args_target_source(captures: regex::Captures) -> Vec<&str> {
        vec![
            captures.name("target").to_owned().unwrap().as_str(),
            captures.name("source").to_owned().unwrap().as_str()
        ]
    }

    fn to_literal(arg: &str) -> Result<u16, ParseError> {
        let base: u32 = if arg.ends_with("H") { 16 }
            else if arg.ends_with("B") { 2 }
            else { 10 };

        let num = if base != 10 {
            &arg[0..arg.len()-1]
        } else {
            &arg
        };

        match u16::from_str_radix(num, base) {
            Ok(literal) => Ok(literal),
            Err(_) => Err(ParseError {
                error_type: ParseErrorType::InvalidNumericLiteral,
                location: 0,
                info: Some(arg.to_string())
            })
        }
    }

    fn to_addressing_mode(arg: &str) -> Result<AddressingMode, ParseError> {
        let arg = arg.to_ascii_uppercase();

        if arg == "XY" {
            Ok(AddressingMode::Index)
        } else {
            match Register::try_from(arg.chars().next().unwrap()) {
                Ok(register) => Ok(AddressingMode::Register(register)),
                Err(_) => {
                    let arg = arg.to_ascii_uppercase();
                    if arg.starts_with("#") {
                        Ok(AddressingMode::Indirect(to_literal(&arg[1..])?))
                    } else {
                        let literal = to_literal(&arg)?;
                        if literal > u8::MAX as u16 {
                            Err(ParseError {
                                error_type: ParseErrorType::InvalidNumericLiteral,
                                location: 0,
                                info: Some(literal.to_string())
                            })
                        } else {
                            Ok(AddressingMode::Immediate(literal as u8))
                        }
                    }
                }
            }
        }
    }

    fn to_instruction(line: &str, location: usize) -> Result<Instruction, ParseError> {
        if let Some(captures) = INST_LOAD.captures(line) {
            let target_source = extract_args_target_source(captures);
            let (target, source) = (target_source[0], target_source[1]);
            let target_register = Register::try_from(target.chars().next().unwrap())?;
            let addressing_mode = to_addressing_mode(source)?;

            Ok(Instruction::Load(target_register, addressing_mode))
        } else if let Some(captures) = INST_STORE.captures(line) {
            let target_source = extract_args_target_source(captures);
            let (target, source) = (target_source[0], target_source[1]);
            let target_register = Register::try_from(target.chars().next().unwrap())?;
            let addressing_mode = to_addressing_mode(source)?;

            Ok(Instruction::Store(target_register, addressing_mode))
        } else if INST_INC.is_match(line) {
            Ok(Instruction::Increment)
        } else if INST_DEC.is_match(line) {
            Ok(Instruction::Decrement)
        } else if let Some(captures) = INST_ADD.captures(line) {
            let source = captures.name("source").to_owned().unwrap().as_str();
            let addressing_mode = to_addressing_mode(source)?;

            Ok(Instruction::Add(addressing_mode))
        } else if let Some(captures) = INST_SUB.captures(line) {
            let source = captures.name("source").to_owned().unwrap().as_str();
            let addressing_mode = to_addressing_mode(source)?;

            Ok(Instruction::Subtract(addressing_mode))
        } else if let Some(captures) = INST_CMP.captures(line) {
            let source = captures.name("source").to_owned().unwrap().as_str();
            let addressing_mode = to_addressing_mode(source)?;

            Ok(Instruction::Compare(addressing_mode))
        } else if let Some(captures) = INST_JUMP.captures(line) {
            let label = captures.name("label").to_owned().unwrap().as_str().to_string();

            Ok(Instruction::_LabeledJump(
                JumpType::Unconditional, label
            ))
        } else if let Some(captures) = INST_CONDITIONAL_JUMP.captures(line) {
            let label = captures.name("label").to_owned().unwrap().as_str().to_string();
            let flag = Flag::try_from(captures.name("flag").to_owned().unwrap().as_str().chars().next().unwrap())?;
            let condition = captures.name("condition").unwrap().as_str().is_empty(); // is_empty <=> 'n' is not present

            Ok(Instruction::_LabeledJump(
                JumpType::Flag(flag, condition), label
            ))
        } else if let Some(captures) = INST_CALL.captures(line) {
            let label = captures.name("label").to_owned().unwrap().as_str().to_string();

            Ok(Instruction::_LabeledCall(label))
        } else if INST_RETURN.is_match(line) {
            Ok(Instruction::Return)
        } else if let Some(captures) = INST_PUSH.captures(line) {
            let source = Register::try_from(captures.name("source").to_owned().unwrap().as_str().chars().next().unwrap())?;

            Ok(Instruction::Push(source))
        } else if let Some(captures) = INST_POP.captures(line) {
            let source = Register::try_from(captures.name("source").to_owned().unwrap().as_str().chars().next().unwrap())?;

            Ok(Instruction::Pop(source))
        } else if INST_NOP.is_match(line) {
            Ok(Instruction::Nop)
        } else if let Some(captures) = INST_LABEL.captures(line) {
            let label = captures.name("label").to_owned().unwrap().as_str().to_string();

            Ok(Instruction::_Label(label))
        } else {
            Err(ParseError {
                error_type: ParseErrorType::InvalidInstruction,
                location,
                info: Some(line.to_string())
            })
        }
    }

    fn preprocess(code: String) -> Vec<(usize, String)> {

        code
            .lines()
            .enumerate()
            .filter_map(|(loc, line)| {
                let trimmed = LEADING_SPACE.replace(line, "");
                let trimmed = DUPLICATE_SPACE.replace(&trimmed, " ");
                let trimmed = COMMENTS.replace(&trimmed, "");
                let trimmed = TRAILING_SPACE.replace(&trimmed, "");

                if BLANK_LINES.is_match(&trimmed) {
                    None
                } else {
                    Some((loc, trimmed.to_string()))
                }
            })
            .collect()
    }
    pub fn compile(code: String) -> ParseResult {
        let preprocessed = preprocess(code);
        let mut output: Vec<Instruction> = Vec::new();
        let mut parse_errors: Vec<ParseError> = Vec::new();

        for (location, line) in preprocessed {
            match to_instruction(&line, location) {
                Ok(instruction) => output.push(instruction),
                Err(error) => parse_errors.push(error)
            }
        }

        if parse_errors.len() > 0 {
            Err(parse_errors)
        } else {
            let processed = super::intermediate::process_labels(output).map_err(
                |missing_labels| {
                    missing_labels.iter().map(|(label, location)| {
                            ParseError {
                                error_type: ParseErrorType::InvalidLabel,
                                location: *location,
                                info: Some(label.clone())
                            }
                    }).collect::<Vec<_>>()
                }
            )?;
            Ok(processed)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::path::Path;

        fn expect_parse_target_source(captures: regex::Captures, expected: Vec<&str>) {
            assert!(
                extract_args_target_source(captures)
                    .iter()
                    .zip(expected)
                    .all(|(cap, exp)| *cap == exp)
            )
        }

        #[test]
        fn test_preprocess() {
            let file = Path::new("sample/square.pop");
            let code = crate::util::read_from_file(file);

            println!("{:?}", preprocess(code));
        }

        #[test]
        fn test_compile_loadstore() {
            let file = Path::new("sample/load_store.pop");
            let code = crate::util::read_from_file(file);

            println!("{:?}", compile(code));
        }

        #[test]
        fn test_compile_squares() {
            let file = Path::new("sample/square.pop");
            let code = crate::util::read_from_file(file);

            println!("{:?}", compile(code));
        }

        #[test]
        fn test_regex_load() {
            expect_parse_target_source(
                INST_LOAD.captures("lda b").unwrap(),
                vec!["a", "b"]
            );
            expect_parse_target_source(
                INST_LOAD.captures("LDA B").unwrap(),
                vec!["A", "B"]
            );
            expect_parse_target_source(
                INST_LOAD.captures("ldb 10").unwrap(),
                vec!["b", "10"]
            );
            expect_parse_target_source(
                INST_LOAD.captures("ldb 10h").unwrap(),
                vec!["b", "10h"]
            );
            expect_parse_target_source(
                INST_LOAD.captures("ldx 10b").unwrap(),
                vec!["x", "10b"]
            );
            assert!(INST_LOAD.captures("lda").is_none());
            assert!(INST_LOAD.captures("lda d").is_none());
            assert!(INST_LOAD.captures("ldl xy").is_none());
        }

        #[test]
        fn test_addressing_parse() {
            assert_eq!(to_addressing_mode("x"), Ok(AddressingMode::Register(Register::IndexX)));
            assert_eq!(to_addressing_mode("X"), Ok(AddressingMode::Register(Register::IndexX)));
            assert_eq!(to_addressing_mode("y"), Ok(AddressingMode::Register(Register::IndexY)));
            assert_eq!(to_addressing_mode("Y"), Ok(AddressingMode::Register(Register::IndexY)));
            assert_eq!(to_addressing_mode("a"), Ok(AddressingMode::Register(Register::Accumulator)));
            assert_eq!(to_addressing_mode("A"), Ok(AddressingMode::Register(Register::Accumulator)));
            assert_eq!(to_addressing_mode("b"), Ok(AddressingMode::Register(Register::UtilityB)));
            assert_eq!(to_addressing_mode("B"), Ok(AddressingMode::Register(Register::UtilityB)));
            assert_eq!(to_addressing_mode("c"), Ok(AddressingMode::Register(Register::UtilityC)));
            assert_eq!(to_addressing_mode("C"), Ok(AddressingMode::Register(Register::UtilityC)));

            assert_eq!(to_addressing_mode("xy"), Ok(AddressingMode::Index));
            assert_eq!(to_addressing_mode("xY"), Ok(AddressingMode::Index));
            assert_eq!(to_addressing_mode("Xy"), Ok(AddressingMode::Index));
            assert_eq!(to_addressing_mode("XY"), Ok(AddressingMode::Index));

            assert_eq!(to_addressing_mode("8"), Ok(AddressingMode::Immediate(8)));
            assert_eq!(to_addressing_mode("1000b"), Ok(AddressingMode::Immediate(0b1000)));
            assert_eq!(to_addressing_mode("10h"), Ok(AddressingMode::Immediate(0x10)));
            assert_eq!(to_addressing_mode("FFh"), Ok(AddressingMode::Immediate(0xFF)));
            assert_eq!(to_addressing_mode("#10h"), Ok(AddressingMode::Indirect(0x10)));
            assert_eq!(to_addressing_mode("#8"), Ok(AddressingMode::Indirect(8)));
            assert_eq!(to_addressing_mode("#FFFFh"), Ok(AddressingMode::Indirect(0xFFFF)));

            // Invalid numbers
            assert_eq!(to_addressing_mode("-"), Err(ParseError {
                error_type: ParseErrorType::InvalidNumericLiteral,
                location: 0,
                info: Some(String::from("-"))
            }));
            assert_eq!(to_addressing_mode("-100h"), Err(ParseError {
                error_type: ParseErrorType::InvalidNumericLiteral,
                location: 0,
                info: Some(String::from("-100H"))
            }));
            // Invalid base
            assert_eq!(to_addressing_mode("2b"), Err(ParseError {
                error_type: ParseErrorType::InvalidNumericLiteral,
                location: 0,
                info: Some(String::from("2B"))
            }));
            // Invalid range
            assert_eq!(to_addressing_mode("FFFFh"), Err(ParseError {
                error_type: ParseErrorType::InvalidNumericLiteral,
                location: 0,
                info: Some(0xFFFF.to_string())
            }));
        }
    }
}

/// A collection of functions that process the intermediate representation `Instruction` of
/// devola assembly
pub mod intermediate {
    use crate::instructions::*;
    use std::collections::HashMap;

    pub type SymbolTable = HashMap<usize, String>;
    pub type ReverseSymbolTable = HashMap<String, usize>;

    pub fn process_labels(code: Vec<Instruction>) -> Result<(Vec<Instruction>, SymbolTable), Vec<(String, usize)>> {
        let jump_table: ReverseSymbolTable = code.iter()
            .enumerate()
            .filter_map(|(pc, instruction)| {
                match instruction {
                    Instruction::_Label(label) => Some((label.clone(), pc)),
                    _ => None
                }
            })
            .collect();

        let mut missing_labels: Vec<(String, usize)> = Vec::new();

        let filtered = code.iter()
            .enumerate()
            .filter_map(|(line, instruction)| {
                match instruction {
                    Instruction::_LabeledJump(jump_type, label) => {
                        if let Some(pc) = jump_table.get(label) {
                            Some(Instruction::Jump(jump_type.clone(), *pc))
                        } else {
                            missing_labels.push((label.clone(), line));
                            None
                        }
                    },
                    Instruction::_LabeledCall(label) => {
                        if let Some(pc) = jump_table.get(label) {
                            Some(Instruction::Call(CallType::Local(*pc)))
                        } else {
                            missing_labels.push((label.clone(), line));
                            None
                        }
                    },
                    Instruction::_Label(_) => Some(Instruction::Nop),
                    _ => Some(instruction.clone())
                }
            })
            .collect();

        if missing_labels.len() > 0 {
            Err(missing_labels)
        } else {
            Ok(
                (filtered,
                 jump_table
                     .iter()
                     .map(
                         |(k, v)| (*v, k.clone())
                     )
                     .collect()
                )
            )
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
                Instruction::_LabeledJump(JumpType::Unconditional, String::from("label")),
                Instruction::_Assert(AddressingMode::Immediate(1), 0),
                Instruction::_Label(String::from("label")),
                Instruction::_Assert(AddressingMode::Register(Register::Accumulator), 10),
            ];
            let code = match process_labels(code) {
                Ok((processed, _)) => processed,
                Err(missing_labels) => {
                    eprintln!("Encountered the following missing labels:");
                    for (label, line) in missing_labels {
                        eprintln!("{} (line {})", label, line + 1);
                    }
                    panic!();
                }
            };

            let mut devola = Devola::new(code, None);
            if let Err(_) = devola.run() {
                panic!();
            }
        }

        #[test]
        fn test_missing_labels() {
            let code: Vec<Instruction> = vec![
                Instruction::_LabeledJump(JumpType::Unconditional, String::from("label")),
                Instruction::_LabeledJump(JumpType::Unconditional, String::from("label2")),
                Instruction::_Label(String::from("label2"))
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
}