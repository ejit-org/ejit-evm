use std::collections::VecDeque;

use ejit::{cpu_info, Cond, CpuInfo, EntryInfo, Executable, Ins, Src, Type, Vsize, R, V};

use revm::{interpreter::Interpreter, primitives::{EVMResult, TxKind}, Database, Evm};

pub struct EjitEvm<'a, EXT, DB: Database> {
    pub(crate) evm: Evm<'a, EXT, DB>,
}

/// Virtual stack, keeps track of items pushed on the stack.
/// 
#[derive(Debug, Clone, Copy)]
pub enum VElem {
    /// A big-endian constant, ie. PUSH4 etc.
    Constant([u8; 32]),

    /// An unknown item passed to this trace on the stack. ie. [bp, #n*32]
    Bp(i32),

    /// In registers
    Reg(u8),
}

impl VElem {
    pub(crate) fn as_c4(&self) -> Option<[u64; 4]> {
        match self {
            VElem::Constant(c) => {
                Some(from_u8(*c))
            }
            _ => None,
        }
    }

    pub(crate) fn as_bp(&self) -> Option<i32> {
        match self {
            VElem::Bp(off) => {
                Some(*off)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct VStack {
    pub(crate) stack: VecDeque<VElem>,
    pub(crate) old_values: usize,
    pub(crate) new_values: usize,
}

#[repr(C)]
#[derive(Debug)]
pub struct InterpreterState {
    pub(crate) mem: * mut u8,
    pub(crate) mem_size: u64,
    pub(crate) gas_remaining: u64,
    pub(crate) contract: [u8; 32],
}

impl InterpreterState {
    pub(crate) fn mem() -> i32 {
        0x00
    }

    pub(crate) fn mem_size() -> i32 {
        0x08
    }

    pub(crate) fn gas() -> i32 {
        0x10
    }
}

impl InterpreterState {
    pub fn new(mem: &mut [u8]) -> Self {
        Self {
            mem: mem.as_mut_ptr(),
            mem_size: mem.len() as u64,
            gas: 0,
        }
    }
}

#[derive(Debug)]
pub struct Compiler {
    pub(crate) ins : Vec<Ins>,
    pub(crate) vstack: VStack,
    pub(crate) constants : Vec<[u64; 4]>,
    pub(crate) cpu_info: CpuInfo,

    // Registers
    pub(crate) t : Box<[R]>,
    pub(crate) bp : R,
    pub(crate) i: R,

    // labels.
    pub(crate) mem_expand: u32,
    pub(crate) revert_overflow: u32,
    pub(crate) revert_gas: u32,
    pub(crate) skip_label: u32,
    pub(crate) mem_expand_overflow: u32,
}

impl Compiler {
    pub(crate) fn new() -> Self {
        let mut cpu_info = ejit::cpu_info();
        let t = (0..8).map(|i| cpu_info.alloc_any().unwrap()).collect();
        let bp = cpu_info.alloc_any().unwrap();
        let i = cpu_info.alloc_any().unwrap();

        let mut c = Self {
            ins: Default::default(),
            vstack: Default::default(),
            constants: Default::default(),
            cpu_info,
            t,
            bp,
            mem_expand: 1000,
            revert_gas: 1001,
            revert_overflow: 1002,
            mem_expand_overflow: 1003,
            skip_label: 10000,
            i,
        };
        // println!("{c:?}");
        c
    }

    pub(crate) fn compile(&mut self, data: &[u8]) {
        use Ins::*;
        let sp = self.cpu_info.sp();
        let bp = self.bp;
        let save_all : Box<[Src]> = self.cpu_info.save().iter().map(Into::into).collect();
        let entry_info = EntryInfo::new()
            .with_saves(self.cpu_info.save())
            .with_args(&[self.i][..])
            .with_res(&[self.t[4], self.t[5]])
            .boxed();
        self.ins.extend([Enter(entry_info.clone()), Mov(bp, sp.into())]);
        let mut pc = 0;
        use revm::interpreter::opcode::*;
        use Ins::*;
        use ejit::Type::*;
        while let Some(&op) = data.get(pc) {
            pc += 1;
            match op {
                PUSH1..=PUSH31  => self.gen_push(data, &mut pc, op),
                ADD => self.gen_add(),
                MSTORE => self.gen_mstore(),
                RETURN => self.gen_return(sp, bp, &entry_info),
                _ => todo!(),
            }
        }
        self.gen_mem_expand_function();
        // for (i, c) in self.constants.iter().enumerate() {
        //     use ejit::Type::*;
        //     self.ins.extend([Label(i as u32), D(U64, c[0]), D(U64, c[1]), D(U64, c[2]), D(U64, c[3])]);
        // }
        for i in &self.ins {
            let indent = match i {
                Label(_) => "",
                _ => "  ",
            };
            println!("{indent}{i:?}")
        }
        let prog = Executable::from_ir(&self.ins).unwrap();
        println!("{}", prog.fmt_url());
        todo!();
    }

    pub(crate) fn gen_return(&mut self, sp: R, bp: R, entry_info: &Box<EntryInfo>) {
        use Ins::*;
        use Type::*;
        use Cond::*;
        let (a, b) = self.vstack.top2();
        let t4 = self.t[4];
        let t5 = self.t[5];
        let t6 = self.t[6];
        self.gen_u64(t4, a);
        self.gen_u64(t5, b);
        self.ins.extend([
            Add(t5, t4, t5.into()),
            Br(Ugt, self.mem_expand_overflow),
        ]);
        self.gen_mem_expand(t5, t6);
        self.ins.extend([
            Ld(U64, t6, self.i, InterpreterState::mem()),
            Add(t5, t5, t6.into()),
            Mov(self.cpu_info.sp(), self.bp.into()),
            Leave(entry_info.clone()),
            Ret,
        ]);
    }

    // https://github.com/ethereum/execution-specs/blob/master/src/ethereum/frontier/vm/instructions/memory.py
    pub(crate) fn gen_mstore(&mut self) {
        use Ins::*;
        use Type::*;
        let t0 = self.t[0];
        let t1 = self.t[1];
        let t2 = self.t[2];
        let t3 = self.t[3];
        let t4 = self.t[4];
        let t5 = self.t[5];
        let (start_position, value) = self.vstack.top2();
        self.gen_t0(start_position);
        self.gen_u64(t4, value);
        self.ins.extend([
            Add(t5, t4, 32.into()),
            Br(Cond::Ugt, self.mem_expand_overflow),
        ]);
        self.gen_mem_expand(t5, self.t[6]);
        self.ins.extend([
            Ld(U64, t5, self.i, InterpreterState::mem()),
            Add(t5, t5, t4.into()),
            St(U64, t3, t4, 0x00),
            St(U64, t2, t4, 0x08),
            St(U64, t1, t4, 0x10),
            St(U64, t0, t4, 0x18),
        ]);
    }

    pub(crate) fn gen_add(&mut self) {
        use Ins::*;
        use Type::*;
        let t0 = self.t[0];
        let t1 = self.t[1];
        let t2 = self.t[2];
        let t3 = self.t[3];
        let t4 = self.t[4];
        let (a, b) = self.vstack.top2();
        if let Some(ca) = a.as_c4() {
            if let Some(cb) = b.as_c4() {
                let sum = add256(ca, cb);
                self.vstack.push(VElem::Constant(to_u8(sum)));
            } else {
                self.gen_t0(b);
                self.ins.extend([
                    Add(t0, t0, ca[0].into()),
                    Adc(t1, t1, ca[1].into()),
                    Adc(t2, t2, ca[2].into()),
                    Adc(t3, t3, ca[3].into()),
                ]);
                self.vstack.push(VElem::Reg(0));
            }
        } else {
            if let Some(cb) = b.as_c4() {
                self.gen_t0(a);
                self.ins.extend([
                    Add(t0, t0, cb[0].into()),
                    Adc(t1, t1, cb[1].into()),
                    Adc(t2, t2, cb[2].into()),
                    Adc(t3, t3, cb[3].into()),
                ]);
                self.vstack.push(VElem::Reg(0));
            } else if let Some(off) = b.as_bp() {
                self.gen_t0(a);
                self.ins.extend([
                    Ld(U64, t4, self.bp, off+0x18),
                    Add(t0, t0, t4.into()),
                    Ld(U64, t4, self.bp, off+0x10),
                    Adc(t1, t1, t4.into()),
                    Ld(U64, t4, self.bp, off+0x08),
                    Adc(t2, t2, t4.into()),
                    Ld(U64, t4, self.bp, off+0x00),
                    Adc(t3, t3, t4.into()),
                ]);
                self.vstack.push(VElem::Reg(0));
            }
        }
    }

    pub(crate) fn gen_push(&mut self, data: &[u8], pc: &mut usize, op: u8) {
        let len = ((op - revm::interpreter::opcode::PUSH1) as usize) + 1;
        if *pc + len > data.len() {
            todo!();
            // generate failure code - must fail at runtime?
        } else {
            let mut c = [0; 32];
            c[32-len..32].copy_from_slice(&data[*pc..*pc+len]);
            self.vstack.push(VElem::Constant(c));
            *pc += len;
        }
    }

    pub(crate) fn gen_u64(&mut self, dest: ejit::R, e: VElem) {
        use Ins::*;
        use Type::*;
        match e {
            VElem::Constant(c) => {
                let c = from_u8(c);
                if c[1] != 0 || c[2] != 0 || c[3] != 0 {
                    self.ins.push(Jmp(self.revert_overflow));
                }
                self.ins.push(Mov(dest, c[0].into()))
            },
            VElem::Bp(imm) => {
                self.ins.extend([
                    Ld(U64, dest, self.bp, imm+0x00),
                    Cmp(dest, 0.into()),
                    Br(ejit::Cond::Ne, self.revert_overflow),
                    Ld(U64, dest, self.bp, imm+0x18),
                ]);
            }
            VElem::Reg(_) => {
                todo!()
            }
        }
    }

    pub(crate) fn gen_mem_expand(&mut self, ptr: ejit::R, mem_size: ejit::R) {
        use ejit::Type::*;
        use ejit::Cond::*;
        use ejit::Ins::*;
        self.ins.extend([
            Ld(U64, mem_size, self.i, InterpreterState::mem_size()),
            Cmp(ptr, mem_size.into()),
            Br(Ule, self.skip_label),
            CallLocal(self.mem_expand),
            Label(self.skip_label),
        ]);
        self.skip_label += 1;
    }

    pub(crate) fn gen_mem_expand_function(&mut self) {
        use ejit::Type::*;
        use ejit::Cond::*;
        use ejit::Ins::*;
        self.ins.extend([
            Label(self.mem_expand),
            Ret,
            Label(self.mem_expand_overflow),
            Ret,
        ]);
        self.skip_label += 1;
    }

    pub(crate) fn gen_t0(&mut self, b: VElem) {
        use ejit::Type::*;
        use ejit::Cond::*;
        use ejit::Ins::*;
        match b {
            VElem::Constant(c) => {
                let c = b.as_c4().unwrap();
                self.ins.extend([
                    Mov(self.t[0], c[0].into()),
                    Mov(self.t[1], c[1].into()),
                    Mov(self.t[2], c[2].into()),
                    Mov(self.t[3], c[3].into()),
                ]);
            }
            VElem::Bp(x) => {
                self.ins.extend([
                    Ld(U64, self.t[0], self.bp, x.into()),
                    Ld(U64, self.t[1], self.bp, x.into()),
                    Ld(U64, self.t[2], self.bp, x.into()),
                    Ld(U64, self.t[3], self.bp, x.into()),
                ]);
            }
            VElem::Reg(0) => {
            }
            _ => todo!(),
        }
    }
}

pub(crate) fn add256(ca: [u64; 4], cb: [u64; 4]) -> [u64; 4] {
    let (sum0, cy0) = ca[0].overflowing_add(cb[0]);

    let (sum1, cy1a) = ca[1].overflowing_add(cb[1]);
    let (sum1, cy1b) = sum1.overflowing_add(if cy0 { 1 } else {0} );

    let (sum2, cy2a) = ca[2].overflowing_add(cb[2]);
    let (sum2, cy2b) = sum2.overflowing_add(if cy1a || cy1b { 1 } else {0} );

    let (sum3, _cy3a) = ca[3].overflowing_add(cb[3]);
    let (sum3, _cy3b) = sum3.overflowing_add(if cy2a || cy2b { 1 } else {0} );

    [sum0, sum1, sum2, sum3]
}

pub(crate) fn sub256(ca: [u64; 4], cb: [u64; 4]) -> [u64; 4] {
    let (sum0, cy0) = ca[0].overflowing_sub(cb[0]);

    let (sum1, cy1a) = ca[1].overflowing_sub(cb[1]);
    let (sum1, cy1b) = sum1.overflowing_sub(if cy0 { 1 } else {0} );

    let (sum2, cy2a) = ca[2].overflowing_sub(cb[2]);
    let (sum2, cy2b) = sum2.overflowing_sub(if cy1a || cy1b { 1 } else {0} );

    let (sum3, _cy3a) = ca[3].overflowing_sub(cb[3]);
    let (sum3, _cy3b) = sum3.overflowing_sub(if cy2a || cy2b { 1 } else {0} );

    [sum0, sum1, sum2, sum3]
}

pub(crate) fn to_u8(ca: [u64; 4]) -> [u8; 32] {
    let mut res = [0; 32];
    res[0x18..0x20].copy_from_slice(&ca[0].to_be_bytes());
    res[0x10..0x18].copy_from_slice(&ca[1].to_be_bytes());
    res[0x08..0x10].copy_from_slice(&ca[2].to_be_bytes());
    res[0x00..0x08].copy_from_slice(&ca[3].to_be_bytes());
    res
}

pub(crate) fn from_u8(c: [u8; 32]) -> [u64; 4] {
    [
        u64::from_be_bytes(c[0x18..0x20].try_into().unwrap()),
        u64::from_be_bytes(c[0x10..0x18].try_into().unwrap()),
        u64::from_be_bytes(c[0x08..0x10].try_into().unwrap()),
        u64::from_be_bytes(c[0x00..0x08].try_into().unwrap()),
    ]
}

impl<'a, EXT, DB: Database> EjitEvm<'a, EXT, DB> {
    pub(crate) fn new(evm: Evm<'a, EXT, DB>) -> Self {
        Self { evm }
    }

    pub(crate) fn transact(&mut self) -> EVMResult<DB::Error> {
        let tx = self.evm.tx();
        let TxKind::Create = tx.transact_to else { todo!() };

        let mut compiler = Compiler::new();
        compiler.compile(&tx.data);

        // let interpreter = InterpreterState::new(contract, gas_limit, is_static);

        todo!()
    }

}

impl VStack {
    pub fn new() -> Self {
        Self { stack: Default::default(), old_values: 0, new_values: 0 }
    }

    /// Ensure at least n items on the vstack.
    pub(crate) fn prep(&mut self, n: usize) {
        while self.stack.len() < n {
            self.stack.push_back(VElem::Bp(self.old_values as i32));
            self.old_values += 32;
        }
    }

    pub(crate) fn top1(&mut self) -> VElem {
        self.prep(1);
        self.stack.pop_front().unwrap()
    }

    pub(crate) fn top2(&mut self) -> (VElem, VElem) {
        self.prep(2);
        (self.stack.pop_front().unwrap(), self.stack.pop_front().unwrap())
    }

    pub(crate) fn top3(&mut self) -> (VElem, VElem, VElem) {
        self.prep(3);
        (self.stack.pop_front().unwrap(), self.stack.pop_front().unwrap(), self.stack.pop_front().unwrap())
    }

    pub(crate) fn push(&mut self, e: VElem) {
        self.stack.push_back(e);
    }
}

pub(crate) mod tests {
    use revm::{db::{CacheDB, EmptyDB}, interpreter, primitives::{Bytecode, ExecutionResult, Output, ResultAndState, TxEnv, TxKind}, Context, Evm, EvmBuilder};

    use crate::EjitEvm;

    #[test]
    fn default() {
        let context = Context::default();
        let db = CacheDB::new(EmptyDB::new());
        let mut tx_env = TxEnv::default();
        tx_env.transact_to = TxKind::Create;
        use interpreter::opcode::*;
        tx_env.data = vec![PUSH1, 2, PUSH1, 0, MSTORE, PUSH1, 0x20, PUSH1, 0, RETURN].into();
        let mut evm = Evm::builder()
            .with_tx_env(tx_env)
            .build();

        let res1 = evm.transact();

        let mut ejit_evm = EjitEvm::new(evm);

        let res2 = ejit_evm.transact();

        let Ok(ResultAndState { result: ExecutionResult::Success { output: Output::Create(out1, addr), .. }, ..}) = res1 else {
            unreachable!()
        };

        let Ok(ResultAndState { result: ExecutionResult::Success { output: Output::Create(out2, addr), .. }, ..}) = res2 else {
            unreachable!()
        };

        todo!();
    }
}
