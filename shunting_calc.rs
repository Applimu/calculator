/*
Todo:
[ ] - Turn some Option< >s into Result< >s 
        - (figure out how Result works in rust)
[X] - Create 'bytecode' to differentiate from tokens
[ ] - Actually implement shunting yard algorithm
*/


type Value = i32;


#[derive(PartialEq,Eq,Clone,Copy,Debug)]
enum Bop {
    ADD,
    MUL,
    SUB,
    MOD,
}

impl Bop {
    fn prec(&self) -> u8 {
        // higher number = put together first
        match self {
            Bop::ADD => 10,
            Bop::MUL => 15,
            Bop::SUB => 10,
            Bop::MOD => 5,
        } 
    }

    fn is_rassoc(&self) -> bool {
        match self {
            Bop::ADD => true,
            Bop::MUL => true,
            Bop::SUB => false,
            Bop::MOD => false,
        }
    }
}

#[derive(Debug)]
enum Token {
    LIT(Value),
    BOP(Bop),
    PAO,
    PAC,
}

#[derive(Debug)]
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

    fn new(stringy:String) -> StrMach {
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

fn getnum(iter: &mut StrMach,base:Value) -> Option<Token> {
    let mut lit: Value = 0;

    loop {
        let n = match iter.next(){
            Some(x) => x,
            None => break,
        };

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

fn lex_str(mut iter: StrMach) -> Vec<Token> {
    

    let mut stack: Vec<Token> = Vec::new();

    
    loop {
        let c = match iter.next() {
            Some(x) => x,
            None => break,
        };

        match c {
            // number literals
            b'b' => stack.push(getnum(&mut iter,2).expect("Failed to parse binary literal")),
            b'd' => stack.push(getnum(&mut iter,10).expect("Failed to parse decimal literal")),
            b'x' => stack.push(getnum(&mut iter,16).expect("Failed to parse hexadecimal literal")),
            b'0'..=b'9' | b'a'..=b'f' => {
                iter.pause();
                stack.push(getnum(&mut iter,10).expect("Failed to parse decimal literal"))
            },
            // binary operations
            b'+' => stack.push(Token::BOP(Bop::ADD)),
            b'*' => stack.push(Token::BOP(Bop::MUL)),
            b'-' => stack.push(Token::BOP(Bop::SUB)),
            b'%' => stack.push(Token::BOP(Bop::MOD)),
            // parenthesis
            b'(' => stack.push(Token::PAO),
            b')' => stack.push(Token::PAC),
            // whitespace handling
            b' '|b'\t' => (),
            b'\n'|b'\r'=> break,
            _ => {
                todo!();
            },

        };
    }

    return stack;
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

fn shunting(tokens: &Vec<Token>) -> Option<Vec<Executable>> {
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
                    match opstack.last() {
                        Some(shuntop) => match shuntop {
                            Shunt::BOP(o2) => {
                               if (o2.prec() <= o1.prec()) && ((o2.prec() != o1.prec()) || o1.is_rassoc())
                                    {break}
                            },
                            Shunt::PAO => break,
                        }
                        None => break,
                    };
                    exestack.push(opstack.pop().unwrap().to_exe());
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
                        None => return None,
                    }
                    mbthis = opstack.pop()
                }
            },
        }
    }
    // clean up the rest of the opstack (until theres nothing left)
    // could probably be a for loop but thats for later
    for this in opstack.iter().rev() {
        match this {
            Shunt::BOP(x) => exestack.push(Executable::BOP(*x)),
            Shunt::PAO => return None, // this means that there is an unclosed set of parens
        }
    }
    Some(exestack)
}

fn eval(stack:&Vec<Executable>) -> Option<Value> {
    //evaluates a stack
    // all tokens *must* be in reverse polish notation 3 5 +
    let mut evalvec:Vec<Value> = Vec::new();
    
    for op in stack.iter() {
        match op {
            Executable::LIT(x) => evalvec.push(*x),
            Executable::BOP(bop) => {
                let tmp_1 = match evalvec.pop() {
                    Some(x) => x,
                    None => {return None},
                };
                let tmp_2 = match evalvec.pop() {
                    Some(x) => x,
                    None => {return None},
                };
                evalvec.push(match bop {
                    Bop::ADD => tmp_1 + tmp_2,
                    Bop::SUB => tmp_2 - tmp_1,
                    Bop::MUL => tmp_1 * tmp_2,
                    Bop::MOD => tmp_2 % tmp_1,
                })
            },
            Executable::NOP => (),
        }
    }
    
    if evalvec.len() == 1 {
        Some(evalvec[0])
    } else {
        None
    }
}

use std::io::{self,Write};
fn main() {
    let mut inp = String::new();
    print!("\x1B[2J");
    loop {
        print!(">> ");
        io::stdout().flush();

        io::stdin()
            .read_line(&mut inp)
            .expect("Didnt get line :(");

        let str_mach = StrMach::new(inp.clone());
        let stack: Vec<Executable> = shunting(&lex_str(str_mach)).expect("Shunted badly :(");

        let output = eval(&stack);

        match output {
            Some(ans) => println!("= {}",ans),
            None => println!("ERROR: failed to evaluate"),
        };
        inp.clear();
    }
}
