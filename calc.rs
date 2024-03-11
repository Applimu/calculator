/*
Features to add:
unary operations:
[ ] - negate
        - (idk how to diff between negation and subtraction)
[ ] - not (!)
[ ] - normalize?? truthy?? idk lets use T

binary operations:
[ ] - and (&&)
[ ] - or (||)
[ ] - xor (#)
[x] - exponentiation (^)
[x] - integer division (//)

modular arithmetic options:
set modulus
modular multiplicative inverse

*/
type Value = i32;

#[allow(non_camel_case_types)]
#[derive(PartialEq,Eq,Clone,Copy,Debug)]
enum Bop {
    ADD,
    MUL,
    SUB,
    MOD,
    DIV,
    EXP,
}

impl Bop {
    fn prec(&self) -> u8 {
        // higher number = put together first
        match self {
            Bop::ADD => 10,
            Bop::MUL => 15,
            Bop::SUB => 10,
            Bop::MOD => 5,
            Bop::DIV => 15,
            Bop::EXP => 20,
        } 
    }

    fn is_rassoc(&self) -> bool {
        match self {
            Bop::ADD => true,
            Bop::MUL => true,
            Bop::SUB => false,
            Bop::MOD => false,
            Bop::DIV => false,
            Bop::EXP => false,
        }
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
enum Token {
    LIT(Value),
    BOP(Bop),
    PAO,
    PAC,
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
enum Executable {
    NOP,
    LIT(Value),
    BOP(Bop),
}

struct StrMach {
    text: Vec<u8>,
    len: usize,
    pos: usize,
    paused: bool,
}

impl StrMach {
    fn from(stringy:String) -> StrMach {
        let text = stringy.into_bytes();
        let length = text.len();
        StrMach {
            text: text,
            len: length,
            pos: 0,
            paused: true,
        }
    }

    fn pause(&mut self) {
        self.paused = true
    }
}

impl Iterator for StrMach {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.paused {
            self.paused = false
        } else {
            self.pos += 1
        };
        
        if self.len < self.pos {
            None
        } else {
            Some(self.text[self.pos])
        }
    }
}

macro_rules! break_if_none {
    ($x:expr) => {
        match $x {
            Some(xpassed) => xpassed,
            None => break,
        }
    };
}


fn getnum(iter: &mut StrMach,base:Value) -> Option<Token> {
    // parses an integer literal when pointing to the first / largest character of the literal
    let mut lit: Value = 0;

    loop {
        // Using a for loop takes possession of it and a ? returns it :/
        let n = break_if_none!(iter.next());

        let digit = match n {
            b'0' => 0,
            b'1' => 1,
            b'2' => 2,
            b'3' => 3,
            b'4' => 4,
            b'5' => 5,
            b'6' => 6,
            b'7' => 7,
            b'8' => 8,
            b'9' => 9,
            b'a' => 10,
            b'b' => 11,
            b'c' => 12,
            b'd' => 14,
            b'e' => 15,
            b'f' => 16,
            _   => break,
        };

        lit = lit * base + digit
    };

    iter.pause();
    Some(Token::LIT(lit))
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
enum LexErr {
    BAD_CHAR(u8),
    FAILED_NUM_LIT,
}


fn lex_str(mut iter: StrMach) -> Result<Vec<Token>,LexErr> {
    

    let mut stack: Vec<Token> = Vec::new();

    
    loop {
        // Using a for loop takes possession of it and a ? returns it :/
        let c = break_if_none!(iter.next());

        match c {
            // number literals
            b'b' => match getnum(&mut iter,2) {
                Some(x) => stack.push(x),
                None => return Err(LexErr::FAILED_NUM_LIT),
            },
            b'd' => match getnum(&mut iter,10) {
                Some(x) => stack.push(x),
                None => return Err(LexErr::FAILED_NUM_LIT),
            },
            b'x' => match getnum(&mut iter,16) {
                Some(x) => stack.push(x),
                None => return Err(LexErr::FAILED_NUM_LIT),
            },
            b'0'..=b'9' => {
                iter.pause();
                match getnum(&mut iter,10) {
                    Some(x) => stack.push(x),
                    None => return Err(LexErr::FAILED_NUM_LIT),
                }
            },
            // binary operations
            b'+' => stack.push(Token::BOP(Bop::ADD)),
            b'*' => stack.push(Token::BOP(Bop::MUL)),
            b'-' => stack.push(Token::BOP(Bop::SUB)),
            b'%' => stack.push(Token::BOP(Bop::MOD)),
            b'/' => {
                if let Some(x) = iter.next() {
                    if x != b'/' {
                        return Err(LexErr::BAD_CHAR(x))
                    }
                } else {
                    return Err(LexErr::BAD_CHAR(b'/'))
                };
                stack.push(Token::BOP(Bop::DIV))
            },
            b'^' => stack.push(Token::BOP(Bop::EXP)),
            // parenthesis
            b'(' => stack.push(Token::PAO),
            b')' => stack.push(Token::PAC),
            // whitespace handling
            b' '|b'\t' => (),
            b'\n'|b'\r'=> break,
            other => {
                return Err(LexErr::BAD_CHAR(other))
            },

        };
    }

    Ok(stack)
}

#[derive(PartialEq,Eq,Debug)]
enum Shunt {
    BOP(Bop),
    PAO,
}

impl Shunt {
    fn to_exe(&self) -> Executable {
        match self {
            Shunt::BOP(o) => Executable::BOP(*o),
            Shunt::PAO => Executable::NOP,
        }
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
enum ParseErr {
    UNCLOSED_PARENS,
}


fn shunting(tokens: &Vec<Token>) -> Result<Vec<Executable>, ParseErr> {
    let mut opstack: Vec<Shunt> = Vec::new();
    let mut exestack: Vec<Executable> = Vec::new();

    for token in tokens.iter() {
        match token {
            Token::LIT(x) => {
                exestack.push(Executable::LIT(*x))
            },
            Token::BOP(op) => {
                let o1 = *op;
                    
                // I am well aware this is a mess and i am sorry
                loop {
                        match break_if_none!(opstack.last()) {
                            Shunt::BOP(o2) => {
                               if (o2.prec() <= o1.prec()) && ((o2.prec() != o1.prec()) || o1.is_rassoc())
                                    {break}
                            },
                            Shunt::PAO => break,
                        }
                    
                    exestack.push(opstack.pop().unwrap().to_exe());
                    // this .pop is guarrenteed to never panic
                }
                opstack.push(Shunt::BOP(o1));
            },
            Token::PAO => {
                opstack.push(Shunt::PAO)
            },
            Token::PAC => {
                let mut mbthis = opstack.pop();
                while mbthis != Some(Shunt::PAO) {
                    match mbthis {
                        Some(this) => exestack.push(this.to_exe()),
                        None => return Err(ParseErr::UNCLOSED_PARENS),
                    }
                    mbthis = opstack.pop()
                }
            },
        }
    }
    // clean up the rest of the opstack (until theres nothing left)
    for this in opstack.iter().rev() {
        match this {
            Shunt::BOP(x) => exestack.push(Executable::BOP(*x)),
            Shunt::PAO => return Err(ParseErr::UNCLOSED_PARENS), // this means that there is an unclosed set of parens
        }
    }
    Ok(exestack)
}

#[allow(non_camel_case_types)]
enum ExecErr {
    EMPTY,
    TOO_FEW_ARGS,
    NONSINGULAR,
    BAD_CALCULATION,
}

fn eval(stack:&Vec<Executable>) -> Result<Value, ExecErr> {
    //evaluates a stack
    // all tokens *must* be in reverse polish notation 3 5 +
    if stack.len() == 0 {
        return Err(ExecErr::EMPTY)
    }
    let mut evalvec:Vec<Value> = Vec::new();
    
    for op in stack.iter() {
        match op {
            Executable::LIT(x) => evalvec.push(*x),
            Executable::BOP(bop) => {
                let b = match evalvec.pop() {
                    Some(x) => x,
                    None => {return Err(ExecErr::TOO_FEW_ARGS)},
                };
                let a = match evalvec.pop() {
                    Some(x) => x,
                    None => {return Err(ExecErr::TOO_FEW_ARGS)},
                };
                evalvec.push(match bop {
                    Bop::ADD => a + b,
                    Bop::SUB => a - b,
                    Bop::MUL => a * b,
                    Bop::MOD => a % b,
                    Bop::DIV => a / b,
                    Bop::EXP => a.pow(b as u32), 
                })
            },
            Executable::NOP => (),
        }
    }
    
    if evalvec.len() == 1 {
        Ok(evalvec[0])
    } else {
        Err(ExecErr::NONSINGULAR)
    }
}

use std::io::{self,Write};
fn main() {
    let mut inp = String::new();
    print!("\x1B[2J");
    loop {
        print!(">> ");
        io::stdout().flush().unwrap();

        io::stdin()
            .read_line(&mut inp)
            .expect("Didnt get line :(");

        let str_mach = StrMach::from(inp.clone());
        let tokenized = lex_str(str_mach).expect("Failed to parse :(");

        let stack: Vec<Executable> = shunting(&tokenized).expect("Shunted badly :(");


        let output = eval(&stack);
        match output {
            Ok(ans) => println!("= {}",ans),
            Err(_) => println!("ERROR: failed to evaluate"),
        };
        inp.clear();
    }
}
