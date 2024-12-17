use crate::compiler::instructions::{Instruction, INSTRUCTION};
use std::collections::HashMap;

fn transpile_to_intermediary_bytecode(source: &str) -> (Vec<String>, HashMap<u8, i32>) {
    let mut lines = source.split('\n').collect::<Vec<&str>>();
    lines.reverse();

    let mut padded_lines = Vec::with_capacity(lines.len());

    // todo: make into parallel iter and map
    for line in lines.iter() {
        let mut symbols = line.split_whitespace().collect::<Vec<&str>>();
        symbols.resize(3, "");

        padded_lines.push(symbols)
    }

    let mut data_label_to_address: HashMap<&str, u8> = HashMap::new();
    let mut address_to_value: HashMap<u8, i32> = HashMap::new();

    let mut cell_labels: HashMap<&str, u8> = HashMap::new();

    for (i, line) in padded_lines.iter().enumerate() {
        let offset = padded_lines.len() as u8 - i as u8 - 1;

        match (line[0], line[1], line[2]) {
            (a, "DAT", b) => match b {
                "" => {
                    address_to_value.insert(offset, 0);
                    data_label_to_address.insert(a, offset)
                }
                _ => {
                    address_to_value.insert(offset, b.parse::<i32>().unwrap());
                    data_label_to_address.insert(a, offset)
                }
            },
            (a, b, _) if INSTRUCTION.contains_key(b) => cell_labels.insert(a, offset),
            _ => None,
        };
    }

    let mut intermediary_bytecode = Vec::new();

    for line in lines.iter().rev() {
        for (i, symbol) in line.split_whitespace().enumerate() {
            if i == 0 && !cell_labels.contains_key(symbol) {
                intermediary_bytecode.push(symbol.to_string());
                continue;
            } else if i == 0 && cell_labels.contains_key(symbol) {
                continue;
            }

            match (
                data_label_to_address.contains_key(symbol),
                   cell_labels.contains_key(symbol),
            ) {
                (true, false) => intermediary_bytecode
                .push((*data_label_to_address.get(symbol).unwrap()).to_string()),
                (false, true) => {
                    intermediary_bytecode.push(cell_labels.get(symbol).unwrap().to_string())
                }
                _ => intermediary_bytecode.push(symbol.to_string()),
            }
        }
    }

    (intermediary_bytecode, address_to_value)
}

fn transpile_to_bytecode(
    intermediary_bytecode_address_to_value_pair: (Vec<String>, HashMap<u8, i32>),
) -> (Vec<Instruction>, HashMap<u8, i32>) {
    let intermediary_bytecode = intermediary_bytecode_address_to_value_pair.0;
    let address_to_value = intermediary_bytecode_address_to_value_pair.1;

    let mut bytecode = Vec::new();

    for (i, symbol) in intermediary_bytecode.iter().enumerate() {
        match INSTRUCTION.contains_key(symbol) {
            true => match symbol.as_str() {
                instruction @ ("ADD" | "SUB") => {
                    let value = intermediary_bytecode[i + 1].parse::<i32>().unwrap();

                    bytecode.push(if instruction == "ADD" {
                        Instruction::Add(value)
                    } else {
                        Instruction::Sub(value)
                    });
                }
                instruction @ ("BRA" | "BRP" | "BRZ" | "LDA" | "STA") => {
                    if let Some(instruction) = match instruction {
                        "BRA" => Some(Instruction::Branch(
                            intermediary_bytecode[i + 1].parse::<u8>().unwrap(),
                        )),
                        "BRP" => Some(Instruction::BranchIfPositive(
                            intermediary_bytecode[i + 1].parse::<u8>().unwrap(),
                        )),
                        "BRZ" => Some(Instruction::BranchIfZero(
                            intermediary_bytecode[i + 1].parse::<u8>().unwrap(),
                        )),
                        "LDA" => Some(Instruction::Load(
                            intermediary_bytecode[i + 1].parse::<u8>().unwrap(),
                        )),
                        "STA" => Some(Instruction::Store(
                            intermediary_bytecode[i + 1].parse::<u8>().unwrap(),
                        )),
                        _ => None,
                    } {
                        bytecode.push(instruction);
                    }
                }
                instruction @ ("HLT" | "INP" | "OUT") => bytecode.push(match instruction {
                    "HLT" => Instruction::Halt,
                    "INP" => Instruction::Input,
                    "OUT" => Instruction::Output,
                    _ => unreachable!(),
                }),
                _ => {}
            },
            false => {}
        }
    }

    (bytecode, address_to_value)
}

fn transpile_to_machine_code(
    bytecode_address_to_value_pair: (Vec<Instruction>, HashMap<u8, i32>),
) -> [i32; 100] {
    let bytecode = bytecode_address_to_value_pair.0;
    let address_to_value = bytecode_address_to_value_pair.1;

    let mut machine_code: Vec<i32> = Vec::with_capacity(100);

    for instruction in bytecode {
        machine_code.push(match instruction {
            Instruction::Add(address) => 100 + address as i32,
            Instruction::Branch(address) => 600 + address as i32,
            Instruction::BranchIfPositive(address) => 800 + address as i32,
            Instruction::BranchIfZero(address) => 700 + address as i32,
            Instruction::Halt => 0,
            Instruction::Input => 901,
            Instruction::Load(address) => 500 + address as i32,
            Instruction::Output => 902,
            Instruction::Store(address) => 300 + address as i32,
            Instruction::Sub(address) => 200 + address as i32,
        });
    }

    machine_code.resize(100, 0);

    // todo: make into parallel iter
    for (address, value) in address_to_value.iter() {
        machine_code[(*address) as usize] = *value;
    }

    println!("{:?}", machine_code);

    TryInto::<[i32; 100]>::try_into(machine_code).unwrap_or([0; 100])
}

pub fn compile(source: &str) -> [i32; 100] {
    transpile_to_machine_code(transpile_to_bytecode(transpile_to_intermediary_bytecode(source)))
}

