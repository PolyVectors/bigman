use phf::phf_map;

#[derive(Copy, Clone, Debug)]
pub enum Instruction {
    Add(i32),
    Branch(u8),
    BranchIfPositive(u8),
    BranchIfZero(u8),
    Halt,
    Input,
    Load(u8),
    Output,
    Store(u8),
    Sub(i32),
}

pub static INSTRUCTION: phf::Map<&'static str, bool> = phf_map! {
    "ADD" => true,
    "BRA" => true,
    "BRP" => true,
    "BRZ" => true,
    "HLT" => true,
    "INP" => true,
    "LDA" => true,
    "OUT" => true,
    "STA" => true,
    "SUB" => true
};
