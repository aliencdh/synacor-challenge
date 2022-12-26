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
        let b = self.mem[self.cur as usize + 1];
        self.cur += 2;

        self.set_register(a, b, self.cur - 2)
    }

    /// Opcode: 2 a
    /// push <a> onto the stack
    pub fn push(&mut self) -> OpcodeResult {
        let a = self.mem[self.cur as usize];
        assert!((a as usize) < MAX_ADDR);
        self.stack.push_back(a);
        self.cur += 1;
        Ok(())
    }

    /// Opcode: 3 a
    /// remove the top element from the stack and write it into <a>; empty stack = error
    pub fn pop(&mut self) -> OpcodeResult {
        let a = self.mem[self.cur as usize];
        assert!((a as usize) < MAX_ADDR);
        let top = self
            .stack
            .pop_back()
            .ok_or(ExecutionError::EmptyStack(self.cur))?;
        self.mem[a as usize] = top;

        Ok(())
    }

    /// Opcode: 4 a b c
    /// set <a> to 1 if <b> is equal to <c>; set it to 0 otherwise
    pub fn eq(&mut self) -> OpcodeResult {
        let a = self.mem[self.cur as usize] as usize;
        assert!(a < MAX_ADDR);
        let b = self.mem[self.cur as usize + 1];
        let c = self.mem[self.cur as usize + 2];

        self.mem[a] = if b == c { 1 } else { 0 };

        self.cur += 3;

        Ok(())
    }

    /// Opcode: 5 a b c
    /// set <a> to 1 if <b> is greater than <c>; set it to 0 otherwise
    pub fn gt(&mut self) -> OpcodeResult {
        let a = self.mem[self.cur as usize] as usize;
        assert!(a < MAX_ADDR);
        let b = self.mem[self.cur as usize + 1];
        let c = self.mem[self.cur as usize + 2];

        self.mem[a] = if b > c { 1 } else { 0 };

        self.cur += 3;

        Ok(())
    }

    /// Opcode: 6 a
    /// jump to <a>
    pub fn jmp(&mut self) -> OpcodeResult {
        let a = self.mem[self.cur as usize];
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
        let a = self.mem[self.cur as usize];
        let b = self.mem[self.cur as usize + 1];

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
        let a = self.mem[self.cur as usize];
        let b = self.mem[self.cur as usize + 1];

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
        let a = self.mem[self.cur as usize] as usize;
        assert!(a < MAX_ADDR);
        // these are usize to avoid overflow
        let b = self.mem[self.cur as usize + 1] as usize;
        let c = self.mem[self.cur as usize + 2] as usize;

        self.mem[a] = ((b + c) % MAX_ADDR) as u16;

        Ok(())
    }

    /// Opcode: 10
    /// store into <a> the product of <b> and <c> (modulo 32768)
    pub fn mult(&mut self) -> OpcodeResult {
        Ok(())
    }

    /// Opcode: 11
    /// store into <a> the remainder of <b> divided by <c>
    pub fn modulo(&mut self) -> OpcodeResult {
        Ok(())
    }

    /// Opcode: 12
    /// stores into <a> the bitwise and of <b> and <c>
    pub fn and(&mut self) -> OpcodeResult {
        Ok(())
    }

    /// Opcode: 13
    /// stores into <a> the bitwise or of <b> and <c>
    pub fn or(&mut self) -> OpcodeResult {
        Ok(())
    }

    /// Opcode: 14
    /// stores 15-bit bitwise inverse of <b> in <a>
    pub fn not(&mut self) -> OpcodeResult {
        Ok(())
    }

    /// Opcode: 15
    /// read memory at address <b> and write it to <a>
    pub fn rmem(&mut self) -> OpcodeResult {
        Ok(())
    }

    /// Opcode: 16
    /// write the value from <b> into memory at address <a>
    pub fn wmem(&mut self) -> OpcodeResult {
        Ok(())
    }

    /// Opcode: 17
    /// write the address of the next instruction to the stack and jump to <a>
    pub fn call(&mut self) -> OpcodeResult {
        Ok(())
    }

    /// Opcode: 18
    /// remove the top element from the stack and jump to it; empty stack = halt
    pub fn ret(&mut self) -> OpcodeResult {
        Ok(())
    }

    /// Opcode: 19 a
    /// Write the character represented by ascii code <a> to the terminal.
    pub fn out(&mut self) -> OpcodeResult {
        print!("{}", self.mem[self.cur as usize] as u8 as char);
        // skip past the arg
        self.cur += 1;
        Ok(())
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
        let mut mem = [0; MAX_ADDR];
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
    fn invalid_address() {
        let mut machine = setup(vec![6, MAX_ADDR as u16]);
        assert_eq!(
            machine.exec_next(),
            Err(ExecutionError::InvalidAddress(MAX_ADDR as u16, 1))
        )
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
        let mut machine = setup(vec![3, 2]);
        machine.stack.push_back(10);
        assert_eq!(machine.exec_next(), Ok(()));
        assert_eq!(machine.cur, 1);
        assert_eq!(machine.mem[2], 10);
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
}
