use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

static LABEL_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Copy, Clone)]
pub enum Label {
    Generated(usize),
    Given(&'static str),
}

impl Label {
    pub fn new() -> Label {
        Label::Generated(LABEL_COUNT.fetch_add(1, Ordering::SeqCst))
    }
}

impl From<&'static str> for Label {
    fn from(string: &'static str) -> Label {
        Label::Given(string)
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Label::*;
        match *self {
            Generated(l) => write!(f, ".L{}", l),
            Given(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Copy, Clone)]
pub enum Register {
    Rax,
    Rbx,
    Rcx,
    Rdx,
    Rsp,
    Rbp,
    Rsi,
    Rdi,
    R8,
    R9,
    Rip,
}

pub fn rax() -> Location {
    Location::Register(Register::Rax)
}

pub fn rbx() -> Location {
    Location::Register(Register::Rbx)
}

pub fn rcx() -> Location {
    Location::Register(Register::Rcx)
}

pub fn rdx() -> Location {
    Location::Register(Register::Rdx)
}

pub fn rsp() -> Location {
    Location::Register(Register::Rsp)
}

pub fn rbp() -> Location {
    Location::Register(Register::Rbp)
}

pub fn rsi() -> Location {
    Location::Register(Register::Rsi)
}

pub fn rdi() -> Location {
    Location::Register(Register::Rdi)
}

pub fn r8() -> Location {
    Location::Register(Register::R8)
}

pub fn r9() -> Location {
    Location::Register(Register::R9)
}

pub fn rip() -> Location {
    Location::Register(Register::Rip)
}

pub fn constant(c: i64) -> Location {
    Location::Constant(c)
}

pub fn deref(loc: Location, offset: i64) -> Location {
    match loc {
        Location::Register(reg) => Location::Memory(reg, offset),
        _ => panic!("Attempted to use constant as memory location"),
    }
}

pub fn relative(loc: Location, label: Label) -> Location {
    match loc {
        Location::Register(reg) => Location::Relative(reg, label),
        _ => panic!("Attempted to use constant as memory location"),
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Register::*;
        write!(f, "%")?;
        match *self {
            Rax => write!(f, "rax"),
            Rbx => write!(f, "rbx"),
            Rcx => write!(f, "rcx"),
            Rdx => write!(f, "rdx"),
            Rsp => write!(f, "rsp"),
            Rbp => write!(f, "rbp"),
            Rsi => write!(f, "rsi"),
            Rdi => write!(f, "rdi"),
            R8 => write!(f, "r8"),
            R9 => write!(f, "r9"),
            Rip => write!(f, "rip"),
        }
    }
}

#[derive(Copy, Clone)]
pub enum Location {
    Constant(i64),
    Register(Register),
    Memory(Register, i64),
    Relative(Register, Label),
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Location::*;
        match *self {
            Constant(c) => write!(f, "${}", c),
            Register(r) => write!(f, "{}", r),
            Memory(r, o) => write!(f, "{}({})", o, r),
            Relative(r, l) => write!(f, "{}({})", l, r),
        }
    }
}

enum Instruction {
    Label(Label),
    Push(Location),
    Pop(Location),
    Not(Location),
    Neg(Location),
    Add(Location, Location),
    Sub(Location, Location),
    Mul(Location, Location),
    Xor(Location, Location),
    Cmp(Location, Location),
    Jmp(Label),
    Je(Label),
    Jge(Label),
    Jne(Label),
    Mov(Location, Location),
    Lea(Location, Location),
    Call(&'static str),
    Ret,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Instruction::*;
        match *self {
            Label(ref label) => write!(f, "\n{}:", label),
            Push(loc) => write!(f, "\n\tpushq {}", loc),
            Pop(loc) => write!(f, "\n\tpopq {}", loc),
            Not(loc) => write!(f, "\n\tnotq {}", loc),
            Neg(loc) => write!(f, "\n\tnegq {}", loc),
            Add(source, target) => write!(f, "\n\taddq {},{}", source, target),
            Sub(source, target) => write!(f, "\n\tsubq {},{}", source, target),
            Mul(source, target) => write!(f, "\n\timulq {},{}", source, target),
            Xor(source, target) => write!(f, "\n\txorq {},{}", source, target),
            Cmp(source, target) => write!(f, "\n\tcmpq {},{}", source, target),
            Jmp(ref label) => write!(f, "\n\tjmp {}", label),
            Je(ref label) => write!(f, "\n\tje {}", label),
            Jge(ref label) => write!(f, "\n\tjge {}", label),
            Jne(ref label) => write!(f, "\n\tjne {}", label),
            Mov(source, target) => write!(f, "\n\tmovq {},{}", source, target),
            Lea(source, target) => write!(f, "\n\tleaq {},{}", source, target),
            Call(name) => write!(f, "\n\tcall {}", name),
            Ret => write!(f, "\n\tret"),
        }
    }
}

pub struct GeneratedCode(String);

impl fmt::Display for GeneratedCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct Code {
    label: Label,
    env: Vec<(String, Location)>,
    allocated: usize,
    asm: Vec<Instruction>,
}

impl Code {
    pub fn new(label: Label) -> Code {
        Code {
            label: label,
            env: vec![],
            allocated: 0,
            asm: vec![],
        }
    }

    pub fn label(mut self, label: Label) -> Code {
        self.asm.push(Instruction::Label(label));
        self
    }

    pub fn push(mut self, loc: Location) -> Code {
        self.asm.push(Instruction::Push(loc));
        self
    }

    pub fn pop(mut self, loc: Location) -> Code {
        self.asm.push(Instruction::Pop(loc));
        self
    }

    pub fn mov(mut self, source: Location, target: Location) -> Code {
        self.asm.push(Instruction::Mov(source, target));
        self
    }

    pub fn lea(mut self, source: Location, target: Location) -> Code {
        self.asm.push(Instruction::Lea(source, target));
        self
    }

    pub fn not(mut self, loc: Location) -> Code {
        self.asm.push(Instruction::Not(loc));
        self
    }

    pub fn neg(mut self, loc: Location) -> Code {
        self.asm.push(Instruction::Neg(loc));
        self
    }

    pub fn add(mut self, source: Location, target: Location) -> Code {
        self.asm.push(Instruction::Add(source, target));
        self
    }

    pub fn sub(mut self, source: Location, target: Location) -> Code {
        self.asm.push(Instruction::Sub(source, target));
        self
    }

    pub fn mul(mut self, source: Location, target: Location) -> Code {
        self.asm.push(Instruction::Mul(source, target));
        self
    }

    pub fn xor(mut self, source: Location, target: Location) -> Code {
        self.asm.push(Instruction::Xor(source, target));
        self
    }

    pub fn cmp(mut self, source: Location, target: Location) -> Code {
        self.asm.push(Instruction::Cmp(source, target));
        self
    }

    pub fn jmp(mut self, label: Label) -> Code {
        self.asm.push(Instruction::Jmp(label));
        self
    }

    pub fn je(mut self, label: Label) -> Code {
        self.asm.push(Instruction::Je(label));
        self
    }

    pub fn jge(mut self, label: Label) -> Code {
        self.asm.push(Instruction::Jge(label));
        self
    }

    pub fn jne(mut self, label: Label) -> Code {
        self.asm.push(Instruction::Jne(label));
        self
    }

    pub fn call(mut self, name: &'static str) -> Code {
        self.asm.push(Instruction::Call(name));
        self
    }

    pub fn ret(mut self) -> GeneratedCode {
        self = self.mov(rbp(), rsp()).pop(rbx());
        if self.allocated > 0 {
            self.asm
                .insert(0, Instruction::Sub(constant(self.allocated as i64), rsp()));
        }
        self.asm.insert(0, Instruction::Mov(rsp(), rbp()));
        self.asm.insert(0, Instruction::Push(rbp()));
        self.asm.insert(0, Instruction::Label(self.label));
        self.asm.push(Instruction::Ret);
        GeneratedCode(format!("{}", self))
    }

    pub fn allocate(&mut self, v: String) -> Location {
        self.allocated += 8;
        let loc = deref(rbp(), -(self.allocated as i64));
        self.env.push((v, loc));
        loc
    }

    pub fn get_env(&self) -> &Vec<(String, Location)> {
        &self.env
    }

    pub fn get(&self, v: String) -> Location {
        for (envv, loc) in self.env.iter().rev() {
            if &v == envv {
                return *loc;
            }
        }
        panic!("Attempted to get unbound variable")
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.asm.iter() {
            write!(f, "{}", line)?;
        }
        Ok(())
    }
}