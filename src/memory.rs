pub trait Addressable {
    fn read(&self, address: u16) -> Option<u8>;
    fn write(&mut self, address: u16, value: u8) -> bool;
    fn read2(&self, address: u16) -> Option<u16> {
        if let Some(x0) = self.read(address) {
            if let Some(x1) = self.read(address + 1) {
                return Some((x0 as u16) | ((x1 as u16) << 8));
            }
        };
        None
    }
    fn write2(&mut self, address: u16, value: u16) -> bool {
        let lower = value & 0xff;
        let upper = (value & 0xff00) >> 8;
        self.write(address, lower as u8) && self.write(address + 1, upper as u8)
    }

    fn copy(&mut self, from: u16, to: u16, n: usize) -> bool {
        for i in 0..n {
            if let Some(x) = self.read(from + i as u16) {
                if !self.write(to + i as u16, x) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

pub struct LinearMemory {
    bytes: Vec<u8>,
    size: usize,
}

impl LinearMemory {
    pub fn new(n: usize) -> Self {
        Self {
            bytes: vec![0; n],
            size: n,
        }
    }
}

impl Addressable for LinearMemory {
    fn read(&self, address: u16) -> Option<u8> {
        if (address as usize) < self.size {
            Some(self.bytes[address as usize])
        } else {
            None
        }
    }

    fn write(&mut self, address: u16, value: u8) -> bool {
        if (address as usize) < self.size {
            self.bytes[address as usize] = value;
            true
        } else {
            false
        }
    }
}
