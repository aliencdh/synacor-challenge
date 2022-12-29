use crate::{ExecutionError, MachineState, OpcodeResult, MAX_ADDR};

impl MachineState {
    /// Opcode: 0
    /// Stop execution and terminate the program.
    pub fn halt(&mut self) -> OpcodeResult {
        Err(ExecutionError::Halt)
    }

    /// Opcode: 1 a b
    /// set register <a> to the value of <b>
    pub fn set(&mut self) -> OpcodeResult {
        let a = self.mem[self.cur as usize] as usize;
        let b = match self.mem[self.cur as usize + 1] {
            val if val < MAX_ADDR as u16 => val,
            val => self.get_register(val as usize, self.cur + 1)?,
        };

        self.cur += 2;
        self.set_register(a, b, self.cur - 2)
    }

    /// Opcode: 2 a
    /// push <a> onto the stack
    pub fn push(&mut self) -> OpcodeResult {
        let a = match self.mem[self.cur as usize] {
            val if val < MAX_ADDR as u16 => val,
            val => self.get_register(val as usize, self.cur)?,
        };
        self.stack.push_back(a);
        self.cur += 1;
        Ok(())
    }

    /// Opcode: 3 a
    /// remove the top element from the stack and write it into <a>; empty stack = error
    pub fn pop(&mut self) -> OpcodeResult {
        let top = self
            .stack
            .pop_back()
            .ok_or(ExecutionError::EmptyStack(self.cur - 1))?;

        self.write(self.mem[self.cur as usize], top, self.cur)
    }

    /// Opcode: 4 a b c
    /// set <a> to 1 if <b> is equal to <c>; set it to 0 otherwise
    pub fn eq(&mut self) -> OpcodeResult {
        let b = match self.mem[self.cur as usize + 1] {
            val if val < MAX_ADDR as u16 => val,
            val => self.get_register(val as usize, self.cur + 1)?,
        };
        let c = match self.mem[self.cur as usize + 2] {
            val if val < MAX_ADDR as u16 => val,
            val => self.get_register(val as usize, self.cur + 2)?,
        };

        self.cur += 3;
        self.write(
            self.mem[self.cur as usize - 3],
            (b == c) as u16,
            self.cur - 3,
        )
    }

    /// Opcode: 5 a b c
    /// set <a> to 1 if <b> is greater than <c>; set it to 0 otherwise
    pub fn gt(&mut self) -> OpcodeResult {
        let b = match self.mem[self.cur as usize + 1] {
            val if val < MAX_ADDR as u16 => val,
            val => self.get_register(val as usize, self.cur + 1)?,
        };
        let c = match self.mem[self.cur as usize + 2] {
            val if val < MAX_ADDR as u16 => val,
            val => self.get_register(val as usize, self.cur + 2)?,
        };

        self.cur += 3;
        self.write(
            self.mem[self.cur as usize - 3],
            (b > c) as u16,
            self.cur - 3,
        )
    }

    /// Opcode: 6 a
    /// jump to <a>
    pub fn jmp(&mut self) -> OpcodeResult {
        let a = match self.mem[self.cur as usize] {
            val if val < MAX_ADDR as u16 => val,
            val => self.get_register(val as usize, self.cur)?,
        };

        self.cur = if a < MAX_ADDR as u16 {
            a
        } else {
            return Err(ExecutionError::InvalidAddress(a, self.cur));
        };

        Ok(())
    }

    /// Opcode: 7 a b
    /// if <a> is nonzero, jump to <b>
    pub fn jmp_true(&mut self) -> OpcodeResult {
        let a = match self.mem[self.cur as usize] {
            val if val < MAX_ADDR as u16 => val,
            val => self.get_register(val as usize, self.cur)?,
        };
        let b = match self.mem[self.cur as usize + 1] {
            val if val < MAX_ADDR as u16 => val,
            val => self.get_register(val as usize, self.cur + 1)?,
        };

        self.cur = if b >= MAX_ADDR as u16 {
            return Err(ExecutionError::InvalidAddress(b, self.cur + 1));
        } else if a != 0 {
            b
        } else {
            self.cur + 2
        };

        Ok(())
    }

    /// Opcode: 8 a b
    /// if <a> is zero, jump to <b>
    pub fn jmp_false(&mut self) -> OpcodeResult {
        let a = match self.mem[self.cur as usize] {
            val if val < MAX_ADDR as u16 => val,
            val => self.get_register(val as usize, self.cur)?,
        };
        let b = match self.mem[self.cur as usize + 1] {
            val if val < MAX_ADDR as u16 => val,
            val => self.get_register(val as usize, self.cur + 1)?,
        };

        self.cur = if b >= MAX_ADDR as u16 {
            return Err(ExecutionError::InvalidAddress(b, self.cur + 1));
        } else if a == 0 {
            b
        } else {
            self.cur + 2
        };

        Ok(())
    }

    /// Opcode: 9 a b c
    /// assign into <a> the sum of <b> and <c> (modulo 32768)
    pub fn add(&mut self) -> OpcodeResult {
        let a = self.mem[self.cur as usize];
        // these are usize to avoid overflow
        let b = match self.mem[self.cur as usize + 1] as usize {
            val if val < MAX_ADDR => val,
            val => self.get_register(val, self.cur + 1)? as usize,
        };

        let c = match self.mem[self.cur as usize + 2] as usize {
            val if val < MAX_ADDR => val,
            val => self.get_register(val, self.cur + 2)? as usize,
        };

        self.cur += 3;
        self.write(a, ((b + c) % MAX_ADDR) as u16, self.cur - 3)
    }

    /// Opcode: 10 a b c
    /// store into <a> the product of <b> and <c> (modulo 32768)
    pub fn mult(&mut self) -> OpcodeResult {
        let a = self.mem[self.cur as usize];
        // these are usize to avoid overflow
        let b = match self.mem[self.cur as usize + 1] as usize {
            val if val < MAX_ADDR => val,
            val => self.get_register(val, self.cur + 1)? as usize,
        };

        let c = match self.mem[self.cur as usize + 2] as usize {
            val if val < MAX_ADDR => val,
            val => self.get_register(val, self.cur + 2)? as usize,
        };

        self.cur += 3;
        self.write(a, ((b * c) % MAX_ADDR) as u16, self.cur - 3)
    }

    /// Opcode: 11 a b c
    /// store into <a> the remainder of <b> divided by <c>
    pub fn modulo(&mut self) -> OpcodeResult {
        let a = self.mem[self.cur as usize];
        let b = match self.mem[self.cur as usize + 1] {
            val if val < MAX_ADDR as u16 => val,
            val => self.get_register(val as usize, self.cur + 1)?,
        };
        let c = match self.mem[self.cur as usize + 2] {
            val if val < MAX_ADDR as u16 => val,
            val => self.get_register(val as usize, self.cur + 2)?,
        };

        self.cur += 3;
        self.write(a, b % c, self.cur - 3)
    }

    /// Opcode: 12 a b c
    /// stores into <a> the bitwise and of <b> and <c>
    pub fn and(&mut self) -> OpcodeResult {
        let a = self.mem[self.cur as usize];
        let b = match self.mem[self.cur as usize + 1] {
            val if val < MAX_ADDR as u16 => val,
            val => self.get_register(val as usize, self.cur + 1)?,
        };
        let c = match self.mem[self.cur as usize + 2] {
            val if val < MAX_ADDR as u16 => val,
            val => self.get_register(val as usize, self.cur + 2)?,
        };

        self.cur += 3;
        self.write(a, b & c, self.cur - 3)
    }

    /// Opcode: 13 a b c
    /// stores into <a> the bitwise or of <b> and <c>
    pub fn or(&mut self) -> OpcodeResult {
        let a = self.mem[self.cur as usize];
        let b = match self.mem[self.cur as usize + 1] {
            val if val < MAX_ADDR as u16 => val,
            val => self.get_register(val as usize, self.cur + 1)?,
        };
        let c = match self.mem[self.cur as usize + 2] {
            val if val < MAX_ADDR as u16 => val,
            val => self.get_register(val as usize, self.cur + 2)?,
        };

        self.cur += 3;
        self.write(a, b | c, self.cur - 3)
    }

    /// Opcode: 14 a b
    /// stores 15-bit bitwise inverse of <b> in <a>
    pub fn not(&mut self) -> OpcodeResult {
        let a = self.mem[self.cur as usize];
        let b = match self.mem[self.cur as usize + 1] {
            val if val < MAX_ADDR as u16 => val,
            val => self.get_register(val as usize, self.cur + 1)?,
        };

        self.cur += 2;
        self.write(a, !b, self.cur - 2)
    }

    /// Opcode: 15 a b
    /// read memory at address <b> and write it to <a>
    pub fn rmem(&mut self) -> OpcodeResult {
        let b = match self.mem[self.cur as usize + 1] {
            val if val < MAX_ADDR as u16 => val,
            val => self.get_register(val as usize, self.cur + 1)?,
        };

        self.mem[self.cur as usize] = self.read(b, self.cur + 1)?;
        self.cur += 2;
        Ok(())
    }

    /// Opcode: 16 a b
    /// write the value from <b> into memory at address <a>
    pub fn wmem(&mut self) -> OpcodeResult {
        let a = self.mem[self.cur as usize];
        let b = match self.mem[self.cur as usize + 1] {
            val if val < MAX_ADDR as u16 => val,
            val => self.get_register(val as usize, self.cur + 1)?,
        };

        self.cur += 2;
        self.write(a, dbg!(self.read(b, self.cur - 1)?), self.cur - 2)
    }

    /// Opcode: 17 a
    /// write the address of the next instruction to the stack and jump to <a>
    pub fn call(&mut self) -> OpcodeResult {
        let next_instr = self.cur + 1;
        self.stack.push_back(next_instr);

        let a = match self.mem[self.cur as usize] {
            val if val < MAX_ADDR as u16 => val,
            val => return Err(ExecutionError::InvalidAddress(val, self.cur)),
        };
        // jump to a
        self.cur = a;

        Ok(())
    }

    /// Opcode: 18
    /// remove the top element from the stack and jump to it; empty stack = halt
    pub fn ret(&mut self) -> OpcodeResult {
        let ret_to = self.stack.pop_back().ok_or(ExecutionError::Halt)?;
        self.cur = if ret_to < MAX_ADDR as u16 {
            ret_to
        } else {
            return Err(ExecutionError::InvalidAddress(ret_to, self.cur));
        };

        Ok(())
    }

    /// Opcode: 19 a
    /// Write the character represented by ascii code <a> to the terminal.
    pub fn char_out(&mut self) -> OpcodeResult {
        let ch = match self.mem[self.cur as usize] {
            val if val < MAX_ADDR as u16 => val,
            val => self.get_register(val as usize, self.cur)?,
        } as u8 as char;

        print!("{ch}");
        // skip past the arg
        self.cur += 1;
        Ok(())
    }

    /// Opcode: 20 a
    /// read a character from the terminal and write its ascii code to <a>
    /// it can be assumed that once input starts, it will continue until a newline is encountered
    /// this means that you can safely read whole lines from the keyboard and trust that they will be fully read
    pub fn char_in(&mut self) -> OpcodeResult {
        use std::io::{stdin, Read};

        let read = stdin()
            .bytes()
            .next()
            .ok_or(ExecutionError::EmptyStdin(self.cur - 1))?
            .map_err(|err| ExecutionError::ReadError(format!("{:?}", err), self.cur - 1))?;

        self.cur += 1;
        self.write(self.mem[self.cur as usize], read as u16, self.cur - 1)
    }

    /// Opcode: 21
    /// Does nothing.
    pub fn no_op(&mut self) -> OpcodeResult {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MAX_ADDR;

    fn setup(overrides: Vec<u16>) -> MachineState {
        let mut mem = Vec::from([0; MAX_ADDR]);
        for (i, v) in overrides.into_iter().enumerate() {
            mem[i] = v;
        }

        MachineState::new(mem)
    }

    #[test]
    fn invalid_opcode() {
        let mut machine = setup(vec![u16::MAX]);
        assert_eq!(
            machine.exec_next(),
            Err(ExecutionError::InvalidOpcode(u16::MAX, 0))
        );
    }

    #[test]
    fn invalid_register() {
        let mut machine = setup(vec![]);
        assert_eq!(
            machine.set_register(0, 0, 0),
            Err(ExecutionError::InvalidRegister(0, 0))
        );
    }

    #[test]
    fn halt() {
        let mut machine = setup(vec![]);
        assert_eq!(machine.exec_next(), Err(ExecutionError::Halt))
    }

    #[test]
    fn set() {
        let mut machine = setup(vec![1, MAX_ADDR as u16, 10]);
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.cur, 3);
        assert_eq!(machine.registers[0], 10);
    }

    #[test]
    fn push() {
        let mut machine = setup(vec![2, 10]);
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.cur, 2);
        assert_eq!(machine.stack.back(), Some(&10));
    }

    #[test]
    fn pop() {
        let mut machine = setup(vec![3]);
        machine.stack.push_back(10);
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.cur, 1);
        assert_eq!(machine.mem[0], 10);
        assert!(machine.stack.is_empty());
    }

    #[test]
    fn eq() {
        // true
        let mut machine = setup(vec![4, 0, 2, 2, 4, 0, 2, 1]);
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.cur, 4);
        assert_eq!(machine.mem[0], 1);

        // false
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.cur, 8);
        assert_eq!(machine.mem[0], 0);
    }

    #[test]
    fn gt() {
        // true
        let mut machine = setup(vec![5, 0, 2, 1, 5, 0, 2, 2]);
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.cur, 4);
        assert_eq!(machine.mem[0], 1);

        // false
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.cur, 8);
        assert_eq!(machine.mem[0], 0);
    }

    #[test]
    fn jmp() {
        let mut machine = setup(vec![6, 10]);
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.cur, 10);
    }

    #[test]
    fn jmp_true() {
        // true
        let mut machine = setup(vec![7, 1, 4, 0, 7, 0, 0]);
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.cur, 4);

        // false
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.cur, 7);
    }

    #[test]
    fn jmp_false() {
        // true
        let mut machine = setup(vec![8, 0, 4, 0, 8, 1, 0]);
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.cur, 4);

        // false
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.cur, 7);
    }

    #[test]
    fn add() {
        let mut machine = setup(vec![9, 0, 2, 2]);
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.mem[0], 4);
        assert_eq!(machine.cur, 4);
    }

    #[test]
    fn mult() {
        let mut machine = setup(vec![10, 0, 2, 3]);
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.mem[0], 6);
        assert_eq!(machine.cur, 4);
    }

    #[test]
    fn modulo() {
        // exact
        let mut machine = setup(vec![11, 0, 6, 3, 11, 0, 5, 3]);
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.mem[0], 0);
        assert_eq!(machine.cur, 4);
        // inexact
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.mem[0], 2);
        assert_eq!(machine.cur, 8);
    }

    #[test]
    fn and() {
        let mut machine = setup(vec![12, 0, 1, 3]);
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.mem[0], 1);
        assert_eq!(machine.cur, 4);
    }

    #[test]
    fn or() {
        let mut machine = setup(vec![13, 0, 1, 2]);
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.mem[0], 3);
        assert_eq!(machine.cur, 4);
    }

    #[test]
    fn not() {
        let mut machine = setup(vec![14, 0, 1]);
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.mem[0], !1);
        assert_eq!(machine.cur, 3);
    }

    #[test]
    fn rmem() {
        let mut machine = setup(vec![15, 0, 0]);
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.mem[1], 15);
        assert_eq!(machine.cur, 3);
    }

    #[test]
    fn wmem() {
        let mut machine = setup(vec![16, 0, 1]);
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.mem[0], 0);
        assert_eq!(machine.cur, 3);
    }

    #[test]
    fn call() {
        let mut machine = setup(vec![17, 0]);
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.stack.back(), Some(&2));
        assert_eq!(machine.cur, 0);
    }

    #[test]
    fn ret() {
        // empty stack
        let mut machine = setup(vec![18, 18]);
        assert_eq!(machine.exec_next(), Err(ExecutionError::Halt));
        // valid case
        machine.stack.push_back(10);
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.cur, 10);
    }

    #[test]
    fn out() {
        let mut machine = setup(vec![19, b'a' as u16]);

        // opcode is recognized
        assert_eq!(machine.exec_next(), Ok(()));

        // opcode moves past arguments
        assert_eq!(machine.cur, 2);
    }

    #[test]
    fn no_op() {
        let initial = setup(vec![21]);
        let mut machine = initial.clone();
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(initial.mem, machine.mem);
        assert_eq!(machine.cur, 1);
    }

    #[test]
    fn mini_program() {
        let mut machine = setup(vec![9, 32768, 32769, 4, 19, 32768]);
        machine.registers[1] = 60;
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.cur, 4);
        assert_eq!(machine.registers[0], 64);
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.cur, 6);
    }
}
