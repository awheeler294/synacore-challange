use std::fs;

use anyhow::{anyhow, Result};
use log::{error, info, trace};

#[derive(Debug)]
enum Token {
    // halt: 0
    //   stop execution and terminate the program
    Halt,

    // set: 1 a b
    //   set register <a> to the value of <b>
    Set(u16, u16),

    // push: 2 a
    //   push <a> onto the stack
    Push(u16),

    // pop: 3 a
    //   remove the top element from the stack and write it into <a>; empty stack = error
    Pop(u16),

    // eq: 4 a b c
    //   set <a> to 1 if <b> is equal to <c>; set it to 0 otherwise
    Eq(u16, u16, u16),

    // gt: 5 a b c
    //   set <a> to 1 if <b> is greater than <c>; set it to 0 otherwise
    Gt(u16, u16, u16),

    // jmp: 6 a
    //   jump to <a>
    Jmp(u16),

    // jt: 7 a b
    //   if <a> is nonzero, jump to <b>
    Jt(u16, u16),

    // jf: 8 a b
    //   if <a> is zero, jump to <b>
    Jf(u16, u16),

    // add: 9 a b c
    //   assign into <a> the sum of <b> and <c> (modulo 32768)
    Add(u16, u16, u16),

    // mult: 10 a b c
    //   store into <a> the product of <b> and <c> (modulo 32768)
    Mult(u16, u16, u16),

    // mod: 11 a b c
    //   store into <a> the remainder of <b> divided by <c>
    Mod(u16, u16, u16),

    // and: 12 a b c
    //   stores into <a> the bitwise and of <b> and <c>
    And(u16, u16, u16),

    // or: 13 a b c
    //   stores into <a> the bitwise or of <b> and <c>
    Or(u16, u16, u16),

    // not: 14 a b
    //   stores 15-bit bitwise inverse of <b> in <a>
    Not(u16, u16),

    // rmem: 15 a b
    //   read memory at address <b> and write it to <a>
    Rmem(u16, u16),

    // wmem: 16 a b
    //   write the value from <b> into memory at address <a>
    Wmem(u16, u16),

    // call: 17 a
    //   write the address of the next instruction to the stack and jump to <a>
    Call(u16),

    // ret: 18
    //   remove the top element from the stack and jump to it; empty stack = halt
    Ret(),

    // out: 19 a
    //   write the character represented by ascii code <a> to the terminal
    Out(u16),

    // in: 20 a
    //   read a character from the terminal and write its ascii code to <a>; it can be assumed that once input starts, it will continue until a newline is encountered; this means that you can safely read whole lines from the keyboard and trust that they will be fully read
    In(u16),

    // noop: 21
    //   no operation
    Noop,
}

impl Token {
    fn parse(input: &[u16]) -> Option<(usize, Self)> {
        let mut pc = 0;
        if let Some(val) = input.get(pc) {
            pc += 1;
            match *val {
                0 => {
                    return Some((pc, Self::Halt));
                }

                1 => {
                    let register = input.get(pc)?;
                    pc += 1;
                    let value = input.get(pc)?;
                    pc += 1;
                    return Some((pc, Self::Set(*register, *value)));
                }

                2 => {
                    let value = input.get(pc)?;
                    pc += 1;

                    return Some((pc, Self::Push(*value)));
                }

                3 => {
                    let destination = input.get(pc)?;
                    pc += 1;

                    return Some((pc, Self::Pop(*destination)));
                }

                4 => {
                    let destination = input.get(pc)?;
                    pc += 1;

                    let lhs = input.get(pc)?;
                    pc += 1;

                    let rhs = input.get(pc)?;
                    pc += 1;

                    return Some((pc, Self::Eq(*destination, *lhs, *rhs)));
                }

                5 => {
                    let destination = input.get(pc)?;
                    pc += 1;

                    let lhs = input.get(pc)?;
                    pc += 1;

                    let rhs = input.get(pc)?;
                    pc += 1;

                    return Some((pc, Self::Gt(*destination, *lhs, *rhs)));
                }

                6 => {
                    let destination = input.get(pc)?;
                    pc += 1;

                    return Some((pc, Self::Jmp(*destination)));
                }

                7 => {
                    let test_val = input.get(pc)?;
                    pc += 1;

                    let destination = input.get(pc)?;
                    pc += 1;

                    return Some((pc, Self::Jt(*test_val, *destination)));
                }

                8 => {
                    let test_val = input.get(pc)?;
                    pc += 1;

                    let destination = input.get(pc)?;
                    pc += 1;

                    return Some((pc, Self::Jf(*test_val, *destination)));
                }

                9 => {
                    let destination = input.get(pc)?;
                    pc += 1;

                    let lhs = input.get(pc)?;
                    pc += 1;

                    let rhs = input.get(pc)?;
                    pc += 1;

                    return Some((pc, Self::Add(*destination, *lhs, *rhs)));
                }

                10 => {
                    let destination = input.get(pc)?;
                    pc += 1;

                    let lhs = input.get(pc)?;
                    pc += 1;

                    let rhs = input.get(pc)?;
                    pc += 1;

                    return Some((pc, Self::Mult(*destination, *lhs, *rhs)));
                }

                11 => {
                    let destination = input.get(pc)?;
                    pc += 1;

                    let lhs = input.get(pc)?;
                    pc += 1;

                    let rhs = input.get(pc)?;
                    pc += 1;

                    return Some((pc, Self::Mod(*destination, *lhs, *rhs)));
                }

                12 => {
                    let destination = input.get(pc)?;
                    pc += 1;

                    let lhs = input.get(pc)?;
                    pc += 1;

                    let rhs = input.get(pc)?;
                    pc += 1;

                    return Some((pc, Self::And(*destination, *lhs, *rhs)));
                }

                13 => {
                    let destination = input.get(pc)?;
                    pc += 1;

                    let lhs = input.get(pc)?;
                    pc += 1;

                    let rhs = input.get(pc)?;
                    pc += 1;

                    return Some((pc, Self::Or(*destination, *lhs, *rhs)));
                }

                14 => {
                    let destination = input.get(pc)?;
                    pc += 1;

                    let value = input.get(pc)?;
                    pc += 1;

                    return Some((pc, Self::Not(*destination, *value)));
                }

                15 => {
                    let destination = input.get(pc)?;
                    pc += 1;

                    let source = input.get(pc)?;
                    pc += 1;

                    return Some((pc, Self::Rmem(*destination, *source)));
                }

                16 => {
                    let destination = input.get(pc)?;
                    pc += 1;

                    let value = input.get(pc)?;
                    pc += 1;

                    return Some((pc, Self::Wmem(*destination, *value)));
                }

                17 => {
                    let destination = input.get(pc)?;
                    pc += 1;

                    return Some((pc, Self::Call(*destination)));
                }

                18 => {
                    return Some((pc, Self::Ret()));
                }

                19 => {
                    let value = input.get(pc)?;
                    pc += 1;

                    return Some((pc, Self::Out(*value)));
                }

                20 => {
                    let destination = input.get(pc)?;
                    pc += 1;

                    return Some((pc, Self::In(*destination)));
                }

                21 => {
                    return Some((pc, Self::Noop));
                }

                _ => {
                    error!("Unknown opcode {val}");

                    return None;
                }
            }
        }

        None
    }
}

const U15_MAX: u16 = 32768;
const NUM_REGISTERS: u16 = 8;

struct Machine {
    pc: usize,
    stack: Vec<u16>,
    registers: [u16; NUM_REGISTERS as usize],
    program: Vec<u16>,
}

impl Machine {
    pub fn new(program: Vec<u16>) -> Self {
        Self {
            pc: 0,
            stack: vec![],
            registers: [0; NUM_REGISTERS as usize],
            program,
        }
    }

    fn run(&mut self) {
        info!("Running program");

        loop {
            trace!("pc: {}", self.pc);
            trace!("instruction: {:?}", self.program.get(self.pc));

            if let Some((pc_delta, token)) = Token::parse(&self.program[self.pc..]) {
                self.pc += pc_delta;
                // dbg!(&token)

                match token {
                    Token::Halt => {
                        break;
                    }

                    Token::Set(register, value) => {
                        dbg!(&token);
                        // self.registers[self.fetch_val(register)] = self.fetch_val(value);
                    }

                    Token::Push(value) => {
                        self.stack.push(value);
                    }

                    Token::Pop(destination) => {
                        dbg!(&token);
                    }

                    Token::Eq(destination, lhs, rhs) => {
                        dbg!(&token);
                    }

                    Token::Gt(destination, lhs, rhs) => {
                        dbg!(&token);
                    }

                    Token::Jmp(destination) => {
                        dbg!(&token);
                        self.pc = destination as usize;
                    }

                    Token::Jt(test_val, destination) => {
                        dbg!(&token);
                        if test_val != 0 {
                            self.pc = destination as usize;
                        }
                    }

                    Token::Jf(test_val, destination) => {
                        dbg!(&token);
                        if test_val == 0 {
                            self.pc = destination as usize;
                        }
                    }

                    Token::Add(destination, lhs, rhs) => {
                        dbg!(&token);
                    }

                    Token::Mult(destination, lhs, rhs) => {
                        dbg!(&token);
                    }

                    Token::Mod(destination, lhs, rhs) => {
                        dbg!(&token);
                    }

                    Token::And(destination, lhs, rhs) => {
                        dbg!(&token);
                    }

                    Token::Or(destination, lhs, rhs) => {
                        dbg!(&token);
                    }

                    Token::Not(destination, value) => {
                        dbg!(&token);
                    }

                    Token::Rmem(destination, source) => {
                        dbg!(&token);
                    }

                    Token::Wmem(destination, value) => {
                        dbg!(&token);
                    }

                    Token::Call(destination) => {
                        dbg!(&token);
                    }

                    Token::Ret() => {
                        dbg!(&token);
                    }

                    Token::Out(arg) => {
                        let val = self.fetch_val(arg);
                        print!(
                            "{}",
                            char::from_u32(val as u32)
                                .expect(&format!("Could not convert {val} to char"))
                        );
                    }

                    Token::In(destination) => {
                        dbg!(&token);
                    }

                    Token::Noop => {
                        continue;
                    }

                    _ => {}
                }
            } else {
                self.pc += 1;
            }
        }
    }

    fn fetch_val(&self, arg: u16) -> u16 {
        if arg < U15_MAX {
            arg
        } else {
            self.registers[(arg - U15_MAX) as usize]
        }
    }
}

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let file_path = "challenge.bin";

    let file_contents = fs::read(file_path)
        .expect(&format!("Could not read file {file_path}"))
        .chunks(2)
        .map(|chunk| (chunk[1] as u16) << 8 | chunk[0] as u16)
        .collect::<Vec<u16>>();

    dbg!(&file_contents);

    let mut machine = Machine::new(file_contents);
    machine.run();

    // dbg!(&file_contents);
}
