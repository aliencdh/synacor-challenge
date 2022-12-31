#![macro_use]
extern crate thiserror;

use std::collections::VecDeque;

use color_eyre::eyre;

mod opcodes;

/// The maximum number that can be used as an address on this machine.
pub const MAX_ADDR: usize = 2usize.pow(15);
pub const REGISTER_COUNT: usize = 8;

/// Represents the state of the machine:
/// - `mem` is its entire memory (RAM)
/// - `cur` is the index of the current operation to be executed
/// - `registers` are the 8 registers specified in the architecture spec.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineState {
    pub mem: Vec<u16>,
    pub cur: u16,
    pub registers: [u16; REGISTER_COUNT],
    pub stack: VecDeque<u16>,
}
impl MachineState {
    pub fn new(mem: Vec<u16>) -> Self {
        Self {
            mem,
            cur: 0,
            registers: [0; REGISTER_COUNT],
            stack: VecDeque::new(),
        }
    }

    pub fn run(&mut self) -> OpcodeResult {
        for _ in 0..MAX_ADDR {
            if let Err(err) = self.exec_next() {
                return Err(err);
            }
        }
        Ok(())
    }

    /// Executes the next operation.
    pub fn exec_next(&mut self) -> eyre::Result<(), ExecutionError> {
        self.cur += 1;
        match self.mem[self.cur as usize - 1] {
            0 => self.halt(),
            1 => self.set(),
            2 => self.push(),
            3 => self.pop(),
            4 => self.eq(),
            5 => self.gt(),
            6 => self.jmp(),
            7 => self.jmp_true(),
            8 => self.jmp_false(),
            9 => self.add(),
            10 => self.mult(),
            11 => self.modulo(),
            12 => self.and(),
            13 => self.or(),
            14 => self.not(),
            15 => self.rmem(),
            16 => self.wmem(),
            17 => self.call(),
            18 => self.ret(),
            19 => self.char_out(),
            21 => self.no_op(),
            op => Err(ExecutionError::InvalidOpcode(op, self.cur - 1)),
        }
    }

    /// Attempts to set a register to the provided value.
    /// If the provided register number is invalid, returns an `ExecutionError`.
    pub fn set_register(&mut self, register: usize, val: u16, pos: u16) -> OpcodeResult {
        self.registers
            .get_mut(
                register
                    .checked_sub(MAX_ADDR)
                    .ok_or(ExecutionError::InvalidRegister(register, pos))?,
            )
            .map(|old| *old = val)
            .ok_or(ExecutionError::InvalidRegister(register, pos))
    }

    /// Attempts to read from a register.
    pub fn get_register(&self, register: usize, pos: u16) -> eyre::Result<u16, ExecutionError> {
        self.registers
            .get(
                register
                    .checked_sub(MAX_ADDR)
                    .ok_or(ExecutionError::InvalidRegister(register, pos))?,
            )
            .map(|&val| val)
            .ok_or(ExecutionError::InvalidRegister(register, pos))
    }

    /// Attempts to write the provided value to a register or a memory address.
    pub fn write(&mut self, write_to: u16, val: u16, pos: u16) -> OpcodeResult {
        if write_to < MAX_ADDR as u16 {
            self.mem[write_to as usize] = val;
            Ok(())
        } else {
            self.set_register(write_to as usize, val, pos)
        }
    }

    /// Attempts to read from a register or a memory address.
    pub fn read(&self, read_from: u16, pos: u16) -> eyre::Result<u16, ExecutionError> {
        if read_from < MAX_ADDR as u16 {
            Ok(self.mem[read_from as usize])
        } else {
            self.get_register(read_from as usize, pos)
        }
    }
}

fn main() -> eyre::Result<()> {
    let data = include_bytes!("../challenge.bin")
        .chunks(2)
        .map(|chunk| u16::from_le_bytes(<[u8; 2]>::try_from(chunk).unwrap()))
        .collect::<Vec<_>>();

    dbg!(&data[590..600]);

    let mut machine = MachineState::new(data);

    match machine.run() {
        Ok(()) | Err(ExecutionError::Halt) => {
            println!("\n\n\nMachine exitted normally.");
            Ok(())
        }
        Err(err) => Err(eyre::eyre!("{:?}", err)),
    }
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    #[error("The program halted.")]
    Halt,
    #[error("Invalid opcode `{0}` at index `{1}`")]
    InvalidOpcode(u16, u16),
    #[error("Tried to access invalid register `{0}` at index `{1}`")]
    InvalidRegister(usize, u16),
    #[error("Tried to pop from an empty stack at index `{0}`")]
    EmptyStack(u16),
    #[error("Tried to access invalid address `{0}` at index `{1}`")]
    InvalidAddress(u16, u16),
    #[error("Tried to read from stdin, which was empty, at index `{0}`")]
    EmptyStdin(u16),
    #[error("Encountered an error while trying to read from stdin at index `{1}`: {0}")]
    ReadError(String, u16),
}

pub type OpcodeResult = eyre::Result<(), ExecutionError>;
