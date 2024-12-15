pub struct Computer {
    accumulator: u32,
    address_register: u8,
    instruction_register: u8,
    program_counter: u8,

    ram: [i32; 100],
}

impl Computer {
    pub fn new() -> Computer {
        Computer {
            accumulator: 0,
            address_register: 0,
            instruction_register: 0,
            program_counter: 0,
            ram: [0; 100],
        }
    }

    pub fn load_program(&mut self, machine_code: [i32; 100]) {
        self.ram = machine_code
    }
}
