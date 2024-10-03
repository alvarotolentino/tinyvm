use crate::memory::*;

#[derive(Debug)]
#[repr(u8)]
pub enum Register {
    A,
    B,
    C,
    M,
    SP,
    PC,
    BP,
    FLAGS,
}

impl Register {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            x if x == Register::A as u8 => Some(Register::A),
            x if x == Register::B as u8 => Some(Register::B),
            x if x == Register::C as u8 => Some(Register::C),
            x if x == Register::M as u8 => Some(Register::M),
            x if x == Register::SP as u8 => Some(Register::SP),
            x if x == Register::PC as u8 => Some(Register::PC),
            x if x == Register::BP as u8 => Some(Register::BP),
            x if x == Register::FLAGS as u8 => Some(Register::FLAGS),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Debug)]
pub enum Op {
    Nop,
    Push(u8),
    PopRegister(Register),
    AddStack,
    AddRegister(Register, Register),
    Mov(Register, Register),
}
impl Op {
    pub fn value(&self) -> u8 {
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
}

fn parse_instruction(ins: u16) -> Result<Op, String> {
    let op = (ins & 0xff) as u8;
    match op {
        x if x == Op::Nop.value() => Ok(Op::Nop),
        x if x == Op::Push(0).value() => {
            let arg = (ins & 0xff00) >> 8;
            Ok(Op::Push(arg as u8))
        }
        x if x == Op::PopRegister(Register::A).value() => {
            let reg = (ins & 0xf00) >> 8;
            if let Some(r) = Register::from_u8(reg as u8) {
                Ok(Op::PopRegister(r))
            } else {
                Err(format!("Unknown register 0x{:X}", reg))
            }
        }
        x if x == Op::AddStack.value() => Ok(Op::AddStack),
        x if x == Op::AddRegister(Register::A, Register::B).value() => {
            Ok(Op::AddRegister(Register::A, Register::B))
        }
        x if x == Op::Mov(Register::A, Register::B).value() => {
            Ok(Op::Mov(Register::A, Register::B))
        }
        _ => Err(format!("Unknown instruction 0x{:X}", op)),
    }
}

pub struct Machine {
    registers: [u16; 8],
    pub memory: Box<dyn Addressable>,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            registers: [0; 8],
            memory: Box::new(LinearMemory::new(8 * 1024)),
        }
    }

    pub fn get_register(&self, reg: Register) -> u16 {
        self.registers[reg as usize]
    }

    pub fn pop(&mut self) -> Result<u16, String> {
        let sp = self.registers[Register::SP as usize] - 2;
        if let Some(v) = self.memory.read2(sp) {
            self.registers[Register::SP as usize] -= 2;
            Ok(v)
        } else {
            Err("Stack underflow".to_string())
        }
    }

    pub fn push(&mut self, value: u16) -> Result<(), String> {
        let sp = self.registers[Register::SP as usize];
        if !self.memory.write2(sp, value) {
            return Err("Stack overflow".to_string());
        }
        self.registers[Register::SP as usize] += 2;
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), String> {
        let pc = self.registers[Register::PC as usize];
        let instruction = self.memory.read2(pc).unwrap();
        self.registers[Register::PC as usize] += 2;

        let op = parse_instruction(instruction)?;
        match op {
            Op::Nop => Ok(()),
            Op::Push(arg) => self.push(arg.into()),
            Op::PopRegister(reg) => {
                let value = self.pop()?;
                self.registers[reg as usize] = value;
                Ok(())
            }
            Op::AddStack => {
                let reg1 = self.pop()?;
                let reg2 = self.pop()?;
                self.push(reg1 + reg2)
            }
            Op::AddRegister(reg1, reg2) => {
                self.registers[reg1 as usize] += self.registers[reg2 as usize];
                Ok(())
            }
            Op::Mov(reg1, reg2) => {
                self.registers[reg1 as usize] = self.registers[reg2 as usize];
                Ok(())
            }
        }
        // Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_instruction() {
        assert!(matches!(parse_instruction(0x0), Ok(Op::Nop)));
        assert!(matches!(parse_instruction(0x1), Ok(Op::Push(0))));
        assert!(matches!(
            parse_instruction(0x2),
            Ok(Op::PopRegister(Register::A))
        ));
        assert!(matches!(parse_instruction(0x3), Ok(Op::AddStack)));
        assert!(matches!(
            parse_instruction(0x4),
            Ok(Op::AddRegister(Register::A, Register::B))
        ));
        assert!(matches!(
            parse_instruction(0x5),
            Ok(Op::Mov(Register::A, Register::B))
        ));
    }

    #[test]
    fn test_push_pop() {
        let mut m = Machine::new();
        m.push(0x1234).unwrap();
        m.push(0x5678).unwrap();
        assert_eq!(m.pop().unwrap(), 0x5678);
        assert_eq!(m.pop().unwrap(), 0x1234);
    }

    #[test]
    fn test_add_stack() {
        let mut m = Machine::new();
        m.push((0x8 << 8) + 0x1).unwrap();
        m.push((0x9 << 8) + 0x1).unwrap();
        m.push(0x3).unwrap();
        m.step().unwrap();
        m.step().unwrap();
        m.step().unwrap();
        assert_eq!(m.pop().unwrap(), 8 + 9);
    }

    #[test]
    fn test_add_register() {
        let mut m = Machine::new();
        m.registers[Register::A as usize] = 0x9;
        m.registers[Register::B as usize] = 0x8;
        m.memory.write(0, 0x4);
        m.step().unwrap();
        assert_eq!(m.get_register(Register::A), 0x8 + 0x9);
    }

    #[test]
    fn test_mov() {
        let mut m = Machine::new();
        m.registers[Register::A as usize] = 0x1234;
        m.registers[Register::B as usize] = 0x5678;
        m.memory.write(0, 0x5);
        m.step().unwrap();
        assert_eq!(m.get_register(Register::A), 0x5678);
    }
}
