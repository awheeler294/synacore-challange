pub const HALT: u16 = 0;
pub const SET: u16 = 1;
pub const PUSH: u16 = 2;
pub const POP: u16 = 3;
pub const EQ: u16 = 4;
pub const GT: u16 = 5;
pub const JMP: u16 = 6;
pub const JT: u16 = 7;
pub const JF: u16 = 8;
pub const ADD: u16 = 9;
pub const MULT: u16 = 10;
pub const MOD: u16 = 11;
pub const AND: u16 = 12;
pub const OR: u16 = 13;
pub const NOT: u16 = 14;
pub const RMEM: u16 = 15;
pub const WMEM: u16 = 16;
pub const CALL: u16 = 17;
pub const RET: u16 = 18;
pub const OUT: u16 = 19;
pub const IN: u16 = 20;
pub const NOOP: u16 = 21;

#[derive(Debug)]
pub enum Token {
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

    Unknown(u16),
}

impl Token {
    /// Parse the next token out of a slice of u16's
    pub fn parse(input: &[u16]) -> Option<Self> {
        let val = input.get(0)?;
        Some(match *val {
            HALT => Self::Halt,

            SET => {
                let register = input.get(1)?;
                let value = input.get(2)?;

                Self::Set(*register, *value)
            }

            PUSH => {
                let value = input.get(1)?;

                Self::Push(*value)
            }

            POP => {
                let destination = input.get(1)?;

                Self::Pop(*destination)
            }

            EQ => {
                let destination = input.get(1)?;
                let lhs = input.get(2)?;
                let rhs = input.get(3)?;

                Self::Eq(*destination, *lhs, *rhs)
            }

            GT => {
                let destination = input.get(1)?;
                let lhs = input.get(2)?;
                let rhs = input.get(3)?;

                Self::Gt(*destination, *lhs, *rhs)
            }

            JMP => {
                let destination = input.get(1)?;

                Self::Jmp(*destination)
            }

            JT => {
                let test_val = input.get(1)?;

                let destination = input.get(2)?;

                Self::Jt(*test_val, *destination)
            }

            JF => {
                let test_val = input.get(1)?;
                let destination = input.get(2)?;

                Self::Jf(*test_val, *destination)
            }

            ADD => {
                let destination = input.get(1)?;
                let lhs = input.get(2)?;
                let rhs = input.get(3)?;

                Self::Add(*destination, *lhs, *rhs)
            }

            MULT => {
                let destination = input.get(1)?;
                let lhs = input.get(2)?;
                let rhs = input.get(3)?;

                Self::Mult(*destination, *lhs, *rhs)
            }

            MOD => {
                let destination = input.get(1)?;
                let lhs = input.get(2)?;
                let rhs = input.get(3)?;

                Self::Mod(*destination, *lhs, *rhs)
            }

            AND => {
                let destination = input.get(1)?;
                let lhs = input.get(2)?;
                let rhs = input.get(3)?;

                Self::And(*destination, *lhs, *rhs)
            }

            OR => {
                let destination = input.get(1)?;
                let lhs = input.get(2)?;
                let rhs = input.get(3)?;

                Self::Or(*destination, *lhs, *rhs)
            }

            NOT => {
                let destination = input.get(1)?;
                let value = input.get(2)?;

                Self::Not(*destination, *value)
            }

            RMEM => {
                let destination = input.get(1)?;
                let source = input.get(2)?;

                Self::Rmem(*destination, *source)
            }

            WMEM => {
                let destination = input.get(1)?;
                let value = input.get(2)?;

                Self::Wmem(*destination, *value)
            }

            CALL => {
                let destination = input.get(1)?;

                Self::Call(*destination)
            }

            RET => Self::Ret(),

            OUT => {
                let value = input.get(1)?;

                Self::Out(*value)
            }

            IN => {
                let destination = input.get(1)?;

                Self::In(*destination)
            }

            NOOP => Self::Noop,

            _ => Self::Unknown(*val),
        })
    }

    /// Number to increment the program counter by to move past this instruction.
    pub fn pc_delta(&self) -> usize {
        match *self {
            Self::Halt => 1,
            Self::Set(_, _) => 3,
            Self::Push(_) => 2,
            Self::Pop(_) => 2,
            Self::Eq(_, _, _) => 4,
            Self::Gt(_, _, _) => 4,
            Self::Jmp(_) => 2,
            Self::Jt(_, _) => 3,
            Self::Jf(_, _) => 3,
            Self::Add(_, _, _) => 4,
            Self::Mult(_, _, _) => 4,
            Self::Mod(_, _, _) => 4,
            Self::And(_, _, _) => 4,
            Self::Or(_, _, _) => 4,
            Self::Not(_, _) => 3,
            Self::Rmem(_, _) => 3,
            Self::Wmem(_, _) => 3,
            Self::Call(_) => 2,
            Self::Ret() => 1,
            Self::Out(_) => 2,
            Self::In(_) => 2,
            Self::Noop => 1,
            Self::Unknown(_) => 1,
        }
    }
}

pub fn parse_16_bit_little_endian(input: &[u8]) -> Vec<u16> {
    input
        .chunks(2)
        .map(|chunk| (chunk[1] as u16) << 8 | chunk[0] as u16)
        .collect::<Vec<u16>>()
}

pub fn decompile(program: &[u16]) -> String {
    let mut output = String::new();

    let mut pc = 0;
    while pc < program.len() {
        if let Some(token) = Token::parse(&program[pc..]) {
            output += &format!("{token:?}\n");

            pc += token.pc_delta();
        } else {
            output += &format!("Error: unable to parse {} at {pc}", program[pc]);

            return output;
        }
    }

    output
}
