use anyhow::{anyhow, Context};
use log::{debug, trace};
use std::{
    collections::VecDeque,
    ops::{Add, Mul},
};

use crate::parse::Token;

const U15_MAX: u16 = 32768;
const REGISTER_OFFSET: u16 = U15_MAX;
const NUM_REGISTERS: u16 = 8;

#[derive(Debug, PartialEq)]
pub enum RunState {
    Continue,
    BufferedOutput(String),
    InuptNeeded,
    Error(String),
    Halt,
}

pub struct Machine {
    run_state: RunState,
    pc: usize,
    stack: Vec<u16>,
    memory: Vec<u16>,
    input_buffer: VecDeque<char>,
    output_buffer: Vec<char>,
}

impl Machine {
    pub fn new(program: Vec<u16>) -> Self {
        let mut memory = program.clone();
        memory.extend(vec![0; (U15_MAX + NUM_REGISTERS) as usize - memory.len()].iter());

        Self {
            run_state: RunState::Continue,
            pc: 0,
            stack: vec![],
            memory,
            input_buffer: VecDeque::with_capacity(256),
            output_buffer: Vec::with_capacity(512),
        }
    }

    pub fn run(&mut self) -> &RunState {
        while *self.run_once() == RunState::Continue {}

        &self.run_state
    }

    pub fn run_once(&mut self) -> &RunState {
        match self.run_state {
            RunState::Halt | RunState::Error(_) => {
                return &self.run_state;
            }

            RunState::InuptNeeded => {
                if self.input_buffer.len() == 0 {
                    return &self.run_state;
                }

                self.run_state = RunState::Continue;
            }

            RunState::BufferedOutput(_) => {
                self.run_state = RunState::Continue;
            }

            RunState::Continue => {}
        };

        debug!("pc: {}", self.pc);
        debug!("instruction: {:?}", self.memory.get(self.pc));

        if let Some(token) = Token::parse(&self.memory[self.pc..]) {
            match token {
                Token::Out(_) => {}
                _ => {
                    if self.output_buffer.len() > 0 {
                        self.run_state = RunState::BufferedOutput(self.flush_output_buffer());
                        return &self.run_state;
                    }
                }
            }

            // dbg!(&token);

            if let Err(e) = self.process_token(token) {
                self.run_state =
                    RunState::Error(format!("Error processing token: {e}, pc: {}", self.pc));
            };
        } else {
            self.run_state = RunState::Error(format!(
                "could not parse instruction at {}: {}",
                self.pc, self.memory[self.pc]
            ));
        }

        return &self.run_state;
    }

    pub fn push_input(&mut self, input: &str) {
        self.input_buffer.extend(input.chars());
    }

    fn process_token(&mut self, token: Token) -> anyhow::Result<()> {
        match token {
            Token::Halt => {
                self.run_state = RunState::Halt;
            }

            Token::Set(register, value) => {
                // dbg!(&token);
                if register >= REGISTER_OFFSET && register < REGISTER_OFFSET + NUM_REGISTERS {
                    self.memory[register as usize] = self.fetch_val(value);

                    self.pc += token.pc_delta();
                } else {
                    return Err(anyhow!(
                        "Set: register argument out of bounds: {register}, token: {token:?}"
                    ));
                }
            }

            Token::Push(value) => {
                // dbg!(&token);
                self.stack.push(self.fetch_val(value));

                self.pc += token.pc_delta();
            }

            Token::Pop(destination) => {
                // dbg!(&token);
                if let Some(value) = self.stack.pop() {
                    // dbg!(value);
                    self.memory[destination as usize] = value;

                    self.pc += token.pc_delta();
                } else {
                    return Err(anyhow!("Attempted to pop empty stack"));
                }
            }

            Token::Eq(destination, lhs, rhs) => {
                // dbg!(&token);
                debug!("Eq: {destination}, {lhs}, {rhs}");
                debug!("    pc: {}", self.pc);
                debug!("    lhs is {}", self.fetch_val(lhs));
                debug!("    rhs is {}", self.fetch_val(rhs));

                if self.fetch_val(lhs) == self.fetch_val(rhs) {
                    debug!("    lhs == rhs, set {destination} to 1");

                    self.memory[destination as usize] = 1
                } else {
                    debug!("    lhs != rhs, set {destination} to 0");
                    self.memory[destination as usize] = 0
                }

                self.pc += token.pc_delta();
            }

            Token::Gt(destination, lhs, rhs) => {
                // dbg!(&token);
                if self.fetch_val(lhs) > self.fetch_val(rhs) {
                    self.memory[destination as usize] = 1
                } else {
                    self.memory[destination as usize] = 0
                }
                self.pc += token.pc_delta();
            }

            Token::Jmp(destination) => {
                // dbg!(&token);
                self.pc = self.fetch_val(destination) as usize;
            }

            Token::Jt(test_val, destination) => {
                // dbg!(&token);
                debug!("Jt: {test_val}, {destination}");
                debug!("    pc: {}", self.pc);
                debug!("    test value is {}", self.fetch_val(test_val));

                if self.fetch_val(test_val) != 0 {
                    debug!("    test value is non-zero, set pc to {destination}");

                    self.pc = self.fetch_val(destination) as usize;
                } else {
                    debug!("    test value is zero, continue");

                    self.pc += token.pc_delta();
                }
            }

            Token::Jf(test_val, destination) => {
                // dbg!(&token);
                debug!("Jf: {test_val}, {destination}");
                debug!("    pc: {}", self.pc);
                debug!("    test value is {}", self.fetch_val(test_val));

                if self.fetch_val(test_val) == 0 {
                    debug!("    test value is zero, set pc to {destination}");

                    self.pc = self.fetch_val(destination) as usize;
                } else {
                    debug!("    test value is non-zero, continue");

                    self.pc += token.pc_delta();
                }
            }

            Token::Add(destination, lhs, rhs) => {
                // dbg!(&token);
                let result =
                    Self::aritmatic_mod_u15(self.fetch_val(lhs), self.fetch_val(rhs), u32::add);
                self.memory[destination as usize] = result;

                self.pc += token.pc_delta();
            }

            Token::Mult(destination, lhs, rhs) => {
                // dbg!(&token);
                let result =
                    Self::aritmatic_mod_u15(self.fetch_val(lhs), self.fetch_val(rhs), u32::mul);
                self.memory[destination as usize] = result;

                self.pc += token.pc_delta();
            }

            Token::Mod(destination, lhs, rhs) => {
                // dbg!(&token);
                let result = self.fetch_val(lhs) % self.fetch_val(rhs);
                self.memory[destination as usize] = result;

                self.pc += token.pc_delta();
            }

            Token::And(destination, lhs, rhs) => {
                // dbg!(&token);
                let result = (self.fetch_val(lhs) & self.fetch_val(rhs)) % U15_MAX;
                self.memory[destination as usize] = result;

                self.pc += token.pc_delta();
            }

            Token::Or(destination, lhs, rhs) => {
                // dbg!(&token);
                let result = (self.fetch_val(lhs) | self.fetch_val(rhs)) % U15_MAX;
                self.memory[destination as usize] = result;

                self.pc += token.pc_delta();
            }

            Token::Not(destination, value) => {
                // dbg!(&token);
                debug!("Not: {destination}, {value}");
                debug!("    pc: {}", self.pc);
                debug!("    value is {}", self.fetch_val(value));

                let result = (!self.fetch_val(value)) % U15_MAX;
                debug!("    result is {result}");

                debug!("    set {destination} to {result}");
                self.memory[destination as usize] = result;

                self.pc += token.pc_delta();
            }

            Token::Rmem(destination, source) => {
                // dbg!(&token);
                debug!("Rmem: {destination}, {source}");
                debug!("    pc: {}", self.pc);

                let source = self.fetch_val(source);

                let value = self.memory[source as usize];
                debug!("    value: {value}");

                debug!("    writing {value} to memory address {destination}");
                self.memory[destination as usize] = value;

                self.pc += token.pc_delta();
            }

            Token::Wmem(destination, value) => {
                // dbg!(&token);
                debug!("Wmem: {destination}, {value}");
                debug!("    pc: {}", self.pc);

                let destination = self.fetch_val(destination);
                debug!("    writing {value} to memory address {destination}");
                self.memory[destination as usize] = self.fetch_val(value);

                self.pc += token.pc_delta();
            }

            Token::Call(destination) => {
                // dbg!(&token);
                debug!("Call: {destination}");
                debug!("    pc: {}", self.pc);
                debug!("    push {} on to the stack", self.pc + token.pc_delta());
                debug!("    set pc to {destination}");

                self.stack.push(self.pc as u16 + token.pc_delta() as u16);

                self.pc = self.fetch_val(destination) as usize;
            }

            Token::Ret() => {
                // dbg!(&token);
                if let Some(destination) = self.stack.pop() {
                    self.pc = destination as usize;
                } else {
                    trace!("ret with empty stack = halt");
                    self.run_state = RunState::Halt;
                }
            }

            Token::Out(arg) => {
                // dbg!(&token);
                let val = self.fetch_val(arg);
                self.output_buffer
                    .push(char::from_u32(val as u32).context("Could not convert {val} to char")?);

                self.pc += token.pc_delta();
            }

            Token::In(destination) => {
                // dbg!(&token);
                if let Some(ch) = self.input_buffer.pop_front() {
                    self.memory[destination as usize] = ch as u16;
                    self.pc += token.pc_delta();
                } else {
                    self.run_state = RunState::InuptNeeded;
                }
            }

            Token::Noop => {
                // dbg!(&token);

                self.pc += token.pc_delta();
            }

            Token::Unknown(_val) => {
                // dbg!(&token);

                return Err(anyhow!(
                    "process_token: Unknown token encountered at {}: {token:?}",
                    self.pc
                ));
            }
        };

        Ok(())
    }

    pub fn flush_output_buffer(&mut self) -> String {
        self.output_buffer.drain(0..).collect::<String>()
    }

    #[allow(dead_code)]
    pub fn registers(&self) -> &[u16] {
        &self.memory[REGISTER_OFFSET as usize..(REGISTER_OFFSET + NUM_REGISTERS) as usize]
    }

    /// If arg is a register address return the contents of that register,
    /// otherwise return arg
    fn fetch_val(&self, arg: u16) -> u16 {
        if arg < REGISTER_OFFSET {
            arg
        } else {
            self.memory[arg as usize]
        }
    }

    fn aritmatic_mod_u15(lhs: u16, rhs: u16, f: fn(u32, u32) -> u32) -> u16 {
        (f(lhs as u32, rhs as u32) % U15_MAX as u32) as u16
    }
}

#[cfg(test)]
mod tests {
    use crate::parse::{
        ADD, AND, CALL, EQ, GT, HALT, IN, JF, JMP, JT, MOD, MULT, NOOP, NOT, OR, OUT, POP, PUSH,
        RET, RMEM, SET, WMEM,
    };

    use super::*;

    #[test]
    fn test_simple_program() {
        #[rustfmt::skip]
        let program = vec![
            // Add contents of register 1 (0) and 4, store the result in resister 0
            ADD, REGISTER_OFFSET, REGISTER_OFFSET + 1, 4,
            // Print the value contained in register 0 (4)
            OUT, REGISTER_OFFSET,
        ];

        let mut machine = Machine::new(program);

        let run_state = machine.run();

        let expected = RunState::BufferedOutput(format!("{}", char::from_u32(4).unwrap()));
        assert_eq!(*run_state, expected);

        assert_eq!(machine.memory[32768], 4);
    }

    #[test]
    fn test_print_a() {
        #[rustfmt::skip]
        let program = vec![
            // Set register 1 to 61
            SET, REGISTER_OFFSET + 1, 61,
            // Add contents of register 1 (61) and 4, store the result in resister 0
            ADD, REGISTER_OFFSET, REGISTER_OFFSET + 1, 4,
            // Print the value contained in register 0 (65)
            OUT, REGISTER_OFFSET,
        ];

        let mut machine = Machine::new(program);

        let run_state = machine.run();

        let expected = RunState::BufferedOutput(String::from("A"));
        assert_eq!(*run_state, expected);

        assert_eq!(machine.memory[32768], 65);
    }

    #[test]
    fn test_print_hello_world() {
        #[rustfmt::skip]
        let program = vec![
            OUT, 'H' as u16,
            OUT, 'e' as u16,
            OUT, 'l' as u16,
            OUT, 'l' as u16,
            OUT, 'o' as u16,
            OUT, ' ' as u16,
            OUT, 'W' as u16,
            OUT, 'o' as u16,
            OUT, 'r' as u16,
            OUT, 'l' as u16,
            OUT, 'd' as u16,
        ];

        let mut machine = Machine::new(program);

        let run_state = machine.run();

        let expected = RunState::BufferedOutput(String::from("Hello World"));
        assert_eq!(*run_state, expected);
    }

    #[test]
    fn test_print_multiple() {
        #[rustfmt::skip]
        let program = vec![
            OUT, 'H' as u16,
            OUT, 'e' as u16,
            OUT, 'l' as u16,
            OUT, 'l' as u16,
            OUT, 'o' as u16,
            NOOP,
            OUT, 'W' as u16,
            OUT, 'o' as u16,
            OUT, 'r' as u16,
            OUT, 'l' as u16,
            OUT, 'd' as u16,
        ];

        let mut machine = Machine::new(program);

        let run_state = machine.run();

        let expected = RunState::BufferedOutput(String::from("Hello"));
        assert_eq!(*run_state, expected);

        let run_state = machine.run();

        let expected = RunState::BufferedOutput(String::from("World"));
        assert_eq!(*run_state, expected);
    }

    #[test]
    fn test_halt() {
        #[rustfmt::skip]
        let program = vec![
            // Set register 1 to 61
            SET, REGISTER_OFFSET, 65,
            // Print the value contained in register 0 (65)
            OUT, REGISTER_OFFSET,
            // Halt
            HALT,
            // Add 1 to contents of register 0. This should not be executed 
            ADD, REGISTER_OFFSET, REGISTER_OFFSET, 1,
            // Print the value contained in register 0 (66). This should not be executed
            OUT, REGISTER_OFFSET,
        ];

        let mut machine = Machine::new(program);

        let run_state = machine.run();

        let expected = RunState::BufferedOutput(String::from("A"));
        assert_eq!(*run_state, expected);

        assert_eq!(machine.memory[32768], 65);
    }

    #[test]
    fn test_set() {
        // Set register 0
        #[rustfmt::skip]
        let program = vec![
            // Set register 0 to 61
            SET, REGISTER_OFFSET, 61,
        ];

        let mut machine = Machine::new(program);

        machine.run();

        assert_eq!(machine.registers(), [61, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(machine.memory[32768], 61);

        // Set register 1
        #[rustfmt::skip]
        let program = vec![
            // Set register 1 to 61
            SET, REGISTER_OFFSET + 1, 61,
        ];

        let mut machine = Machine::new(program);

        machine.run();

        assert_eq!(machine.registers(), [0, 61, 0, 0, 0, 0, 0, 0]);
        assert_eq!(machine.memory[32768 + 1], 61);

        // Set register 2
        #[rustfmt::skip]
        let program = vec![
            // Set register 2 to 61
            SET, REGISTER_OFFSET + 2, 61,
        ];

        let mut machine = Machine::new(program);

        machine.run();

        assert_eq!(machine.registers(), [0, 0, 61, 0, 0, 0, 0, 0]);
        assert_eq!(machine.memory[32768 + 2], 61);

        // Set register 3
        #[rustfmt::skip]
        let program = vec![
            // Set register 3 to 61
            SET, REGISTER_OFFSET + 3, 61,
        ];

        let mut machine = Machine::new(program);

        machine.run();

        assert_eq!(machine.registers(), [0, 0, 0, 61, 0, 0, 0, 0]);
        assert_eq!(machine.memory[32768 + 3], 61);

        // Set register 4
        #[rustfmt::skip]
        let program = vec![
            // Set register 4 to 61
            SET, REGISTER_OFFSET + 4, 61,
        ];

        let mut machine = Machine::new(program);

        machine.run();

        assert_eq!(machine.registers(), [0, 0, 0, 0, 61, 0, 0, 0]);
        assert_eq!(machine.memory[32768 + 4], 61);

        // Set register 5
        #[rustfmt::skip]
        let program = vec![
            // Set register 5 to 61
            SET, REGISTER_OFFSET + 5, 61,
        ];

        let mut machine = Machine::new(program);

        machine.run();

        assert_eq!(machine.registers(), [0, 0, 0, 0, 0, 61, 0, 0]);
        assert_eq!(machine.memory[32768 + 5], 61);

        // Set register 6
        #[rustfmt::skip]
        let program = vec![
            // Set register 6 to 61
            SET, REGISTER_OFFSET + 6, 61,
        ];

        let mut machine = Machine::new(program);

        machine.run();

        assert_eq!(machine.registers(), [0, 0, 0, 0, 0, 0, 61, 0]);
        assert_eq!(machine.memory[32768 + 6], 61);

        // Set register 7
        #[rustfmt::skip]
        let program = vec![
            // Set register 7 to 61
            SET, REGISTER_OFFSET + 7, 61,
        ];

        let mut machine = Machine::new(program);

        machine.run();

        assert_eq!(machine.registers(), [0, 0, 0, 0, 0, 0, 0, 61]);
        assert_eq!(machine.memory[32768 + 7], 61);

        // Attempt to set an invalid register 8
        #[rustfmt::skip]
        let program = vec![
            // INVALID: Set register 8 to 61
            SET, REGISTER_OFFSET + 8, 61,
        ];

        let mut machine = Machine::new(program);

        machine.run();

        assert_eq!(machine.registers(), [0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(machine.memory.get(32768 + 8), None);

        // Attempt to set an invalid register -1
        #[rustfmt::skip]
        let program = vec![
            // INVALID: Set register -1 to 61
            SET, REGISTER_OFFSET - 1, 61,
        ];

        let mut machine = Machine::new(program);

        machine.run();

        assert_eq!(machine.registers(), [0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(machine.memory.get(32768 + 8), None);
    }
}
