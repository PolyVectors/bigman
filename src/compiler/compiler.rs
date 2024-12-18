use crate::compiler::instructions::{Instruction, INSTRUCTION};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rayon::prelude::*;

fn transpile_to_intermediary_bytecode(
    source: &str,
) -> (Arc<Mutex<Vec<String>>>, Arc<Mutex<HashMap<u8, i32>>>) {
    let lines = source.split('\n').collect::<Vec<&str>>();

    let padded_lines = lines
        .par_iter()
        .map(|line| {
            let mut symbols = line.split_whitespace().collect::<Vec<&str>>();
            symbols.resize(3, "");

            symbols
        })
        .collect::<Vec<_>>();

    let (data_label_to_address, address_to_value, cell_labels) = (
        Arc::new(Mutex::from(HashMap::new())),
        Arc::new(Mutex::from(HashMap::new())),
        Arc::new(Mutex::from(HashMap::new())),
    );

    let (
        data_label_to_address_for_closures,
        address_to_value_for_closures,
        cell_labels_for_closures,
    ) = (
        Arc::clone(&data_label_to_address),
        Arc::clone(&address_to_value),
        Arc::clone(&cell_labels),
    );

    padded_lines
        .par_iter()
        .rev()
        .enumerate()
        .for_each(|(i, line)| {
            let (mut data_label_to_address, mut address_to_value, mut cell_labels) = (
                data_label_to_address_for_closures.lock().unwrap(),
                address_to_value_for_closures.lock().unwrap(),
                cell_labels_for_closures.lock().unwrap(),
            );

            let offset = padded_lines.len() as u8 - i as u8 - 1;

            match (line[0], line[1], line[2]) {
                (a, "DAT", b) => {
                    if b == "" {
                        address_to_value.insert(offset, 0);
                        data_label_to_address.insert(a, offset)
                    } else {
                        address_to_value.insert(offset, b.parse::<i32>().unwrap());
                        data_label_to_address.insert(a, offset)
                    }
                }
                (a, b, _) if INSTRUCTION.contains_key(b) => cell_labels.insert(a, offset),
                _ => None,
            };
        });

    let intermediary_bytecode = Arc::new(Mutex::from(Vec::new()));

    let unsorted_intermediary_bytecode: Vec<(usize, Vec<String>)> = lines
        .par_iter()
        .enumerate()
        .map(|(i, line)| {
            let (data_label_to_address, cell_labels) = (
                data_label_to_address_for_closures.lock().unwrap(),
                cell_labels_for_closures.lock().unwrap(),
            );

            let mut intermediary_bytecode = Vec::new();

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

            (i, intermediary_bytecode)
        })
        .collect();

    for (_, line) in unsorted_intermediary_bytecode {
        intermediary_bytecode.lock().unwrap().extend(line)
    }

    (intermediary_bytecode, address_to_value)
}


fn transpile_to_bytecode(
    intermediary_bytecode_address_to_value_pair: (Arc<Mutex<Vec<String>>>, Arc<Mutex<HashMap<u8, i32>>>),
) -> (Arc<Mutex<Vec<Instruction>>>, Arc<Mutex<HashMap<u8, i32>>>) {
    let intermediary_bytecode = intermediary_bytecode_address_to_value_pair.0;
    let address_to_value = intermediary_bytecode_address_to_value_pair.1;

    let bytecode = Arc::new(Mutex::from(Vec::new()));
    let (intermediary_bytecode_for_closure, bytecode_for_closure) = (Arc::clone(&intermediary_bytecode).lock().unwrap().clone(), Arc::clone(&bytecode));

    for (i, symbol) in intermediary_bytecode_for_closure.iter().enumerate() {
        let mut bytecode = bytecode_for_closure.lock().unwrap(); 
        
        match INSTRUCTION.contains_key(symbol) {
            true => match symbol.as_str() {
                instruction @ ("ADD" | "SUB") => {
                    let value = intermediary_bytecode_for_closure[i + 1].parse::<i32>().unwrap();

                    bytecode.push(if instruction == "ADD" {
                        Instruction::Add(value)
                    } else {
                        Instruction::Sub(value)
                    });
                }
                instruction @ ("BRA" | "BRP" | "BRZ" | "LDA" | "STA") => {
                    if let Some(instruction) = match instruction {
                        "BRA" => Some(Instruction::Branch(
                            intermediary_bytecode_for_closure[i + 1].parse::<u8>().unwrap(),
                        )),
                        "BRP" => Some(Instruction::BranchIfPositive(
                            intermediary_bytecode_for_closure[i + 1].parse::<u8>().unwrap(),
                        )),
                        "BRZ" => Some(Instruction::BranchIfZero(
                            intermediary_bytecode_for_closure[i + 1].parse::<u8>().unwrap(),
                        )),
                        "LDA" => Some(Instruction::Load(
                            intermediary_bytecode_for_closure[i + 1].parse::<u8>().unwrap(),
                        )),
                        "STA" => Some(Instruction::Store(
                            intermediary_bytecode_for_closure[i + 1].parse::<u8>().unwrap(),
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
    };

    (bytecode, address_to_value)
}

fn transpile_to_machine_code(
    bytecode_address_to_value_pair: (Arc<Mutex<Vec<Instruction>>>, Arc<Mutex<HashMap<u8, i32>>>),
) -> [i32; 100] {
    let bytecode = bytecode_address_to_value_pair.0;
    let address_to_value = bytecode_address_to_value_pair.1;

    let machine_code: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::from(Arc::clone(&bytecode)
        .lock()
        .unwrap()
        .clone()
        .into_par_iter()
        .map(|instruction| {
            return match instruction {
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
            }
        })
        .collect::<Vec<i32>>()));

    machine_code.lock().unwrap().resize(100, 0);

    address_to_value.lock().unwrap().clone().into_par_iter().for_each(|(address, value)| {
        Arc::clone(&machine_code).lock().unwrap()[address as usize] = value;
    });

    let machine_code = machine_code.lock().unwrap().clone();
    TryInto::<[i32; 100]>::try_into(machine_code).unwrap_or([0; 100])
}

pub fn compile(source: &str) -> [i32; 100] {
    transpile_to_machine_code(transpile_to_bytecode(transpile_to_intermediary_bytecode(source)))
}

