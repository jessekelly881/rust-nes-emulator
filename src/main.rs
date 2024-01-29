type Address = u16;
type Value = u8;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NonAddressing,
}

#[derive(Debug)]
pub struct CPU {
    pub register_a: Value,
    pub register_x: Value,
    pub register_y: Value,
    pub status: Value,
    pub program_counter: u16,
    memory: [Value; 0xFFFF],
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: [0; 0xFFFF],
        }
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_op_address(mode);
        let value = self.mem_read(addr);

        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a)
    }

    fn inx(&mut self) {
        if self.register_x == 0xff {
            self.register_x = 0;
        } else {
            self.register_x += 1;
        }

        self.update_zero_and_negative_flags(self.register_x);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x)
    }

    fn get_op_address(&self, mode: &AddressingMode) -> Address {
        match mode {
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }

            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }

            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }

            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);

                let ptr = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);

                (hi as u16) << 8 | (lo as u16)
            }

            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);

                deref
            }

            AddressingMode::NonAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    fn mem_read(&self, addr: Address) -> Value {
        self.memory[addr as usize]
    }

    fn mem_read_u16(&self, addr: Address) -> u16 {
        let lo = self.mem_read(addr) as u16;
        let hi = self.mem_read(addr + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write(&mut self, addr: Address, value: Value) {
        self.memory[addr as usize] = value;
    }

    fn mem_write_u16(&mut self, addr: Address, value: u16) {
        let hi = (value >> 8) as u8;
        let lo = (value & 0xff) as u8;
        self.mem_write(addr, lo);
        self.mem_write(addr + 1, hi)
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0;

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<Value>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn load_and_run(&mut self, program: Vec<Value>) {
        self.load(program);
        self.reset();
        self.run();
    }

    fn update_zero_and_negative_flags(&mut self, result: Value) {
        // is zero?
        if result == 0 {
            self.status = self.status | 0b0000_0010; // set zero flag
        } else {
            self.status = self.status & 0b1111_1101; // unset zero flag
        }

        // is negative?
        if result & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000; // set negative flag
        } else {
            self.status = self.status & 0b0111_1111; // unset negative flag
        }
    }

    pub fn run(&mut self) {
        loop {
            let opscode = self.mem_read(self.program_counter);
            self.program_counter += 1;

            match opscode {
                // BRK
                0x00 => return,

                // LDA _
                0xA9 => {
                    self.lda(&AddressingMode::Immediate);
                    self.program_counter += 1; // pass over param
                }

                0xA5 => {
                    self.lda(&AddressingMode::ZeroPage);
                    self.program_counter += 1; // pass over param
                }

                0xAD => {
                    self.lda(&AddressingMode::Absolute);
                    self.program_counter += 1; // pass over param
                }

                0xAA => self.tax(),
                0xE8 => self.inx(),
                _ => todo!(""),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mem_read() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xa9, 0x05, 0x00]);

        assert_eq!(cpu.mem_read(0x8000), 0xa9);
        assert_eq!(cpu.mem_read(0x8001), 0x05);

        assert_eq!(cpu.mem_read_u16(0x8000), 0x05a9)
    }

    #[test]
    fn test_0xa9_lda_load() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);

        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00); // zero flag not set
        assert!(cpu.status & 0b1000_0000 == 0) // negative flag not set
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]); // load 5; break;
        assert!(cpu.status & 0b0000_0010 == 0b10); // zero flag set
    }

    #[test]
    fn test_0xaa_tax() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0xaa, 0x00]); // load 5; tax; break;

        assert_eq!(cpu.register_x, 0x05)
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.register_x = 0xff;

        // lda 0xff
        // tax
        // inx
        // inx
        // break
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }

    #[test]
    fn test_lda_from_memory() {
        // pointer
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55); // load data at 0x10
        cpu.load_and_run(vec![0xa5, 0x10, 0x00]); // read data at 0x10

        assert_eq!(cpu.register_a, 0x55);
    }
}

fn main() {
    let mut cpu = CPU::new();
    println!("{:?}", cpu);
}
