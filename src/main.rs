#[derive(Debug)]
pub struct CPU {
    pub register_a: u8,
    pub status: u8,
    pub program_counter: u16,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            status: 0,
            program_counter: 0,
        }
    }

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.program_counter = 0;

        loop {
            let opscode = program[self.program_counter as usize];
            self.program_counter += 1;

            match opscode {
                // BRK
                0x00 => return,

                // LDA _
                0xA9 => {
                    let param = program[self.program_counter as usize];
                    self.program_counter += 1; // pass over param
                    self.register_a = param;

                    // is zero?
                    if self.register_a == 0 {
                        self.status = self.status | 0b0000_0010; // set zero flag
                    } else {
                        self.status = self.status & 0b1111_1101; // unset zero flag
                    }

                    // is negative?
                    if self.register_a & 0b1000_0000 != 0 {
                        self.status = self.status | 0b1000_0000; // set negative flag
                    } else {
                        self.status = self.status & 0b0111_1111; // unset negative flag
                    }
                }

                _ => todo!(""),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_load() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x05, 0x00]); // load 5; break;

        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00); // zero flag not set
        assert!(cpu.status & 0b1000_0000 == 0) // negative flag not set
    }

    #[test]
    fn test_0xa9_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x00, 0x00]); // load 5; break;
        assert!(cpu.status & 0b0000_0010 == 0b10); // zero flag set
    }
}

fn main() {
    let mut cpu = CPU::new();
    println!("{:?}", cpu);
}
