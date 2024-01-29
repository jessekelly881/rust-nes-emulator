#[derive(Debug)]
pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub status: u8,
    pub program_counter: u16,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            status: 0,
            program_counter: 0,
        }
    }

    fn lda(&mut self, value: u8) {
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

    fn update_zero_and_negative_flags(&mut self, result: u8) {
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
                    self.lda(param)
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
    fn test_0xa9_lda_load() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x05, 0x00]); // load 5; break;

        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00); // zero flag not set
        assert!(cpu.status & 0b1000_0000 == 0) // negative flag not set
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x00, 0x00]); // load 5; break;
        assert!(cpu.status & 0b0000_0010 == 0b10); // zero flag set
    }

    #[test]
    fn test_0xaa_tax() {
        let mut cpu = CPU::new();
        cpu.register_a = 10;
        cpu.interpret(vec![0xaa, 0x00]); // tax; break;

        assert_eq!(cpu.register_x, 10)
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.register_x = 0xff;
        cpu.interpret(vec![0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }
}

fn main() {
    let mut cpu = CPU::new();
    println!("{:?}", cpu);
}
