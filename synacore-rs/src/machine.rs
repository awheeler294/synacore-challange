use anyhow::{anyhow, Context};
use log::{error, info, trace};

use crate::parse::Token;

pub trait OutputBuffer {
    fn push(&mut self, val: char);
    fn flush(&mut self);
    fn len(&self) -> usize;
    fn contents(&self) -> &[char];
}

const U15_MAX: u16 = 32768;
const REGISTER_OFFSET: u16 = U15_MAX;
const NUM_REGISTERS: u16 = 8;

pub struct Machine {
    run: bool,
    pc: usize,
    stack: Vec<u16>,
    memory: Vec<u16>,
    output_buffer: Box<dyn OutputBuffer>,
}

impl Machine {
    pub fn new(program: Vec<u16>, output_buffer: Box<dyn OutputBuffer>) -> Self {
        let mut memory = program.clone();
        memory.extend(vec![0; (U15_MAX + NUM_REGISTERS) as usize - memory.len()].iter());

        Self {
            run: false,
            pc: 0,
            stack: vec![],
            memory,
            output_buffer,
        }
    }

    pub fn run(&mut self) {
        self.run = true;
        // let mut flush = false;
        info!("Running program");

        while self.run {
            trace!("pc: {}", self.pc);
            trace!("instruction: {:?}", self.memory.get(self.pc));

            if let Some(token) = Token::parse(&self.memory[self.pc..]) {
                match token {
                    Token::Out(_) => {}
                    _ => {
                        if self.output_buffer.len() > 0 {
                            self.output_buffer.flush();
                        }
                    }
                }

                // dbg!(&token);

                if let Err(e) = self.process_token(token) {
                    error!("Error processing token: {e}, pc: {}", self.pc);
                    return;
                };
            } else {
                error!("could not process instruction at {}: {}", self.pc, self.memory[self.pc]);
                self.pc += 1;
            }
        }
    }

    fn process_token(&mut self, token: Token) -> anyhow::Result<()> {
        match token {
            Token::Halt => {
                self.run = false;
            }

            Token::Set(register, value) => {
                // dbg!(&token);
                if register >= REGISTER_OFFSET && register < REGISTER_OFFSET + NUM_REGISTERS {
                    self.memory[register as usize] = self.fetch_val(value);
                
                    self.pc += token.pc_delta();
                } else {
                    self.run = false;
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
                    self.memory[destination as usize] = value;

                    self.pc += token.pc_delta();
                } else {
                    self.run = false;
                    return Err(anyhow!("Attempted to pop empty stack"));
                }
            }

            Token::Eq(destination, lhs, rhs) => {
                // dbg!(&token);
                dbg!(&self.pc);
                dbg!(&self.fetch_val(destination));
                dbg!(&self.fetch_val(lhs));
                dbg!(&self.fetch_val(rhs));
                if self.fetch_val(lhs) == self.fetch_val(rhs) {
                    self.memory[destination as usize] = 1
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
                dbg!(&token);
                self.pc = self.fetch_val(destination) as usize;
            }

            Token::Jt(test_val, destination) => {
                dbg!(&token);
                dbg!(&self.pc);
                dbg!(self.fetch_val(test_val));
                if self.fetch_val(test_val) != 0 {
                    self.pc = self.fetch_val(destination) as usize;
                } else {
                    self.pc += token.pc_delta();
                }
            }

            Token::Jf(test_val, destination) => {
                dbg!(&token);
                if self.fetch_val(test_val) == 0 {
                    self.pc = self.fetch_val(destination) as usize;
                } else {
                    self.pc += token.pc_delta();
                }
            }

            Token::Add(destination, lhs, rhs) => {
                // dbg!(&token);
                let result = self.fetch_val(lhs) + self.fetch_val(rhs);
                self.memory[destination as usize] = result;

                self.pc += token.pc_delta();
            }

            Token::Mult(destination, lhs, rhs) => {
                // dbg!(&token);
                let result = self.fetch_val(lhs) * self.fetch_val(rhs);
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
                let result = self.fetch_val(lhs) & self.fetch_val(rhs);
                self.memory[destination as usize] = result;

                self.pc += token.pc_delta();
            }

            Token::Or(destination, lhs, rhs) => {
                // dbg!(&token);
                let result = self.fetch_val(lhs) | self.fetch_val(rhs);
                self.memory[destination as usize] = result;

                self.pc += token.pc_delta();
            }

            Token::Not(destination, value) => {
                // dbg!(&token);
                let result = !self.fetch_val(value);
                self.memory[destination as usize] = result;

                self.pc += token.pc_delta();
            }

            Token::Rmem(destination, source) => {
                // dbg!(&token);
                let value = self.memory[source as usize];
                self.memory[destination as usize] = value;

                self.pc += token.pc_delta();
            }

            Token::Wmem(destination, value) => {
                // dbg!(&token);
                self.memory[destination as usize] = self.fetch_val(value);

                self.pc += token.pc_delta();
            }

            Token::Call(destination) => {
                dbg!(&token);
                dbg!(self.pc + 2);
                dbg!(self.fetch_val(destination));

                self.stack.push(self.pc as u16 + 2);

                self.pc = self.fetch_val(destination) as usize;
            }

            Token::Ret() => {
                // dbg!(&token);
                if let Some(destination) = self.stack.pop() {
                    self.pc = destination as usize;
                } else {
                    trace!("ret with empty stack = halt");
                    self.run = false;
                }
            }

            Token::Out(arg) => {
                // dbg!(&token);
                let val = self.fetch_val(arg);
                self.output_buffer
                    .push(char::from_u32(val as u32).context("Could not convert {val} to char")?);

                self.pc += token.pc_delta();
            }

            Token::In(_destination) => {
                // dbg!(&token);

                self.pc += token.pc_delta();
            }

            Token::Noop => {
                // dbg!(&token);

                self.pc += token.pc_delta();
            }

            Token::Unknown(_val) => {
                // dbg!(&token);

                self.run = false;

                return Err(anyhow!(
                    "process_token: Unknown token encountered at {}: {token:?}",
                    self.pc
                ));
            }
        };

        Ok(())
    }

    fn fetch_val(&self, arg: u16) -> u16 {
        if arg < REGISTER_OFFSET {
            arg
        } else {
            self.memory[arg as usize]
        }
    }

    pub fn flush_output_buffer(&mut self) {
        self.output_buffer.flush();
    }

    #[allow(dead_code)]
    pub fn registers(&self) -> &[u16] {
        &self.memory[REGISTER_OFFSET as usize..(REGISTER_OFFSET + NUM_REGISTERS) as usize]
    }
}

#[cfg(test)]
mod tests {
    use crate::parse::{
        ADD, AND, CALL, EQ, GT, HALT, IN, JF, JMP, JT, MOD, MULT, NOOP, NOT, OR, OUT, POP, PUSH,
        RET, RMEM, SET, WMEM,
    };

    use super::*;

    struct TestOutputBuffer {
        buff: Vec<char>,
    }

    impl TestOutputBuffer {
        fn new() -> Self {
            Self {
                buff: Vec::with_capacity(1024),
            }
        }
    }

    impl OutputBuffer for TestOutputBuffer {
        fn push(&mut self, val: char) {
            self.buff.push(val);
        }

        fn flush(&mut self) {}

        fn len(&self) -> usize {
            self.buff.len()
        }

        fn contents(&self) -> &[char] {
            &self.buff[0..]
        }
    }

    #[test]
    fn test_simple_program() {
        #[rustfmt::skip]
        let program = vec![
            // Add contents of register 1 (0) and 4, store the result in resister 0
            ADD, REGISTER_OFFSET, REGISTER_OFFSET + 1, 4,
            // Print the value contained in register 0 (4)
            OUT, REGISTER_OFFSET,
        ];

        let mut machine = Machine::new(program, Box::new(TestOutputBuffer::new()));

        machine.run();

        assert_eq!(machine.memory[32768], 4);
        assert_eq!(
            machine.output_buffer.contents(),
            [char::from_u32(4).unwrap()]
        );
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

        let mut machine = Machine::new(program, Box::new(TestOutputBuffer::new()));

        machine.run();

        assert_eq!(machine.memory[32768], 65);
        assert_eq!(machine.output_buffer.contents(), ['A']);
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

        let mut machine = Machine::new(program, Box::new(TestOutputBuffer::new()));

        machine.run();

        assert_eq!(machine.memory[32768], 65);
        assert_ne!(machine.output_buffer.contents(), ['A', 'B']);
        assert_eq!(machine.output_buffer.contents(), ['A']);
    }

    #[test]
    fn test_set() {
        // Set register 0
        #[rustfmt::skip]
        let program = vec![
            // Set register 0 to 61
            SET, REGISTER_OFFSET, 61,
        ];

        let mut machine = Machine::new(program, Box::new(TestOutputBuffer::new()));

        machine.run();

        assert_eq!(machine.registers(), [61, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(machine.memory[32768], 61);

        // Set register 1
        #[rustfmt::skip]
        let program = vec![
            // Set register 1 to 61
            SET, REGISTER_OFFSET + 1, 61,
        ];

        let mut machine = Machine::new(program, Box::new(TestOutputBuffer::new()));

        machine.run();

        assert_eq!(machine.registers(), [0, 61, 0, 0, 0, 0, 0, 0]);
        assert_eq!(machine.memory[32768 + 1], 61);

        // Set register 2
        #[rustfmt::skip]
        let program = vec![
            // Set register 2 to 61
            SET, REGISTER_OFFSET + 2, 61,
        ];

        let mut machine = Machine::new(program, Box::new(TestOutputBuffer::new()));

        machine.run();

        assert_eq!(machine.registers(), [0, 0, 61, 0, 0, 0, 0, 0]);
        assert_eq!(machine.memory[32768 + 2], 61);

        // Set register 3
        #[rustfmt::skip]
        let program = vec![
            // Set register 3 to 61
            SET, REGISTER_OFFSET + 3, 61,
        ];

        let mut machine = Machine::new(program, Box::new(TestOutputBuffer::new()));

        machine.run();

        assert_eq!(machine.registers(), [0, 0, 0, 61, 0, 0, 0, 0]);
        assert_eq!(machine.memory[32768 + 3], 61);

        // Set register 4
        #[rustfmt::skip]
        let program = vec![
            // Set register 4 to 61
            SET, REGISTER_OFFSET + 4, 61,
        ];

        let mut machine = Machine::new(program, Box::new(TestOutputBuffer::new()));

        machine.run();

        assert_eq!(machine.registers(), [0, 0, 0, 0, 61, 0, 0, 0]);
        assert_eq!(machine.memory[32768 + 4], 61);

        // Set register 5
        #[rustfmt::skip]
        let program = vec![
            // Set register 5 to 61
            SET, REGISTER_OFFSET + 5, 61,
        ];

        let mut machine = Machine::new(program, Box::new(TestOutputBuffer::new()));

        machine.run();

        assert_eq!(machine.registers(), [0, 0, 0, 0, 0, 61, 0, 0]);
        assert_eq!(machine.memory[32768 + 5], 61);

        // Set register 6
        #[rustfmt::skip]
        let program = vec![
            // Set register 6 to 61
            SET, REGISTER_OFFSET + 6, 61,
        ];

        let mut machine = Machine::new(program, Box::new(TestOutputBuffer::new()));

        machine.run();

        assert_eq!(machine.registers(), [0, 0, 0, 0, 0, 0, 61, 0]);
        assert_eq!(machine.memory[32768 + 6], 61);

        // Set register 7
        #[rustfmt::skip]
        let program = vec![
            // Set register 7 to 61
            SET, REGISTER_OFFSET + 7, 61,
        ];

        let mut machine = Machine::new(program, Box::new(TestOutputBuffer::new()));

        machine.run();

        assert_eq!(machine.registers(), [0, 0, 0, 0, 0, 0, 0, 61]);
        assert_eq!(machine.memory[32768 + 7], 61);

        // Attempt to set an invalid register 8
        #[rustfmt::skip]
        let program = vec![
            // INVALID: Set register 8 to 61
            SET, REGISTER_OFFSET + 8, 61,
        ];

        let mut machine = Machine::new(program, Box::new(TestOutputBuffer::new()));

        machine.run();

        assert_eq!(machine.registers(), [0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(machine.memory.get(32768 + 8), None);

        // Attempt to set an invalid register -1
        #[rustfmt::skip]
        let program = vec![
            // INVALID: Set register -1 to 61
            SET, REGISTER_OFFSET - 1, 61,
        ];

        let mut machine = Machine::new(program, Box::new(TestOutputBuffer::new()));

        machine.run();

        assert_eq!(machine.registers(), [0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(machine.memory.get(32768 + 8), None);
    }
}
