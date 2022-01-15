// Copyright 2022 The Psyche Authors
// This file is part of Psyche.
//
// Psyche is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Psyche is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with Psyche. If not, see <http://www.gnu.org/licenses/>.

use crate::instructions::EvmOpcode;
use crate::schedule::Schedule;
use crate::vm::VmRom;
use crate::vm::BbInfo;
use crate::u256::U256;

use std::collections::{HashMap};
use std::fmt;

type Lifetime = (isize, isize, u16, bool, i16);

#[derive(Debug)]
struct Instr {
    opcode: EvmOpcode,
    args: Vec<Argument>,
    sp_offset: i16,
}

struct InstrWithConsts<'a> {
    instr: &'a Instr,
    consts: &'a [U256],
}

struct InstrWithConstsAndLifetimes<'a> {
    instr: &'a Instr,
    consts: &'a [U256],
    lifetimes: &'a [Lifetime],
}

impl Instr {
    fn with_consts<'a>(instr: &'a Instr, consts: &'a [U256]) -> InstrWithConsts<'a> {
        InstrWithConsts {
            instr,
            consts,
        }
    }

    fn with_consts_and_lifetimes<'a>(instr: &'a Instr, consts: &'a [U256], lifetimes: &'a [Lifetime]) -> InstrWithConstsAndLifetimes<'a> {
        InstrWithConstsAndLifetimes {
            instr,
            consts,
            lifetimes,
        }
    }

    fn invalid() -> Instr {
       Instr {
            opcode: EvmOpcode::INVALID,
            args: vec!(),
            sp_offset: 0,
        }
    }

    fn new(
        opcode: EvmOpcode,
        retarg: Option<Argument>,
        args: &[Argument],
    ) -> Instr {
        let mut v = vec![];
        if let Some(arg) = retarg {
            v.push(arg);
        }
        for a in args {
            v.push(*a);
        }
        Instr {
            opcode,
            args: v,
            sp_offset: 0,
        }
    }

    fn set2(dst0: Argument, dst1: Argument, src0: Argument, src1: Argument) -> Instr {
        let mut v = vec![];
        v.push(dst0);
        v.push(dst1);
        v.push(src0);
        v.push(src1);
        Instr {
            opcode: EvmOpcode::SWAP1,
            args: v,
            sp_offset: 0,
        }
    }
}

impl<'a> fmt::Display for InstrWithConsts<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self.instr.opcode.to_string().to_lowercase();
        let s = if s == "swap1" {
            "set2".to_string()
        } else {
            s
        };
        let res = write!(f, "{:<7} ", s);

        for arg in self.instr.args.iter() {
            match arg {
                Argument::Immediate { value } => {
                    write!(f, "${}, ", value.0[0]);
                },
                Argument::Input { id: _, address } => {
                    write!(f, "@{:+}, ", address);
                },
                Argument::Temporary { id } => {
                    write!(f, "r{}, ", id);
                },
            }
        }
        let sp_offset = self.instr.sp_offset;
        if sp_offset != 0 {
            write!(f, "({:+})", sp_offset);
        }
        res
    }
}

impl<'a> fmt::Display for InstrWithConstsAndLifetimes<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self.instr.opcode.to_string().to_lowercase();
        let res = write!(f, "{} ", s);

        for arg in self.instr.args.iter() {
            match arg {
                Argument::Immediate { value } => {
                    write!(f, "${}, ", value.0[0]);
                },
                Argument::Input { id: _, address } => {
                    write!(f, "@{}, ", address);
                },
                Argument::Temporary { id } => {
                    let res = self.lifetimes.iter().find(|&tu| tu.2 == *id);
                    let (_,_,_,_,addr) = res.unwrap();
                    write!(f, "@{}, ", addr);
                },
            }
        }
        let sp_offset = self.instr.sp_offset;
        if sp_offset != 0 {
            write!(f, "({:+})", sp_offset);
        }
        res
    }
}

#[derive(Debug, Copy, Clone)]
enum Argument {
    Immediate { value: U256 },
    Input { id: u16, address: i16 },
    Temporary { id: u16 },
}

struct StaticStack {
    size: usize,
    args: Vec<Argument>,
    rcs: HashMap<u16, usize>,
    lifetimes: HashMap<u16, (isize, Option<isize>)>,
    next_id: u16,
}

impl StaticStack {
    fn new() -> StaticStack {
        const CAPACITY: usize = 1024;
        StaticStack {
            size: 0,
            args: Vec::with_capacity(CAPACITY),
            rcs: HashMap::with_capacity(CAPACITY),
            lifetimes: HashMap::with_capacity(CAPACITY),
            next_id: 0,
        }
    }

    fn len(&self) -> usize {
        self.args.len()
    }

    fn size(&self) -> usize {
        self.size
    }

    fn clear(&mut self, size: usize) {
        self.size = size;
        self.args.clear();
        self.rcs.clear();
        self.lifetimes.clear();
        self.next_id = 0;

        for i in 0..size {
            let address = (i as isize - size as isize) as i16;

            self.push(Argument::Input { id: self.next_id, address }, -1);
            //println!("push i{} @{}", self.next_id, address);
            self.next_id += 1;
        }
    }

    fn push(&mut self, arg: Argument, pc: isize) -> &mut Self {
        let id = match arg {
            Argument::Input { id, address: _ } => Some(id),
            Argument::Temporary { id } => Some(id),
            Argument::Immediate { value: _ } => None,
        };
        if let Some(id) = id {
            if let Some(v) = self.rcs.get_mut(&id) {
                // increment refcount
                *v = *v + 1;
            } else {
                // it's a new argument, insert with refcount that is set to 1
                self.rcs.insert(id, 1);
                //println!("inserting {} at {}", id, pc);
                self.lifetimes.insert(id, (pc as isize, None));
            }
        }
        //println!("pushing on the stack {:?}", arg);
        self.args.push(arg);
        self
    }

    fn pop(&mut self, pc: isize) -> (&mut Self, Argument) {
        //println!("len: {}", self.args.len());
        // TODO: explain why it's safe to unwrap
        let arg = self.args.pop().unwrap();
        let id = match arg {
            Argument::Input { id, address: _ } => Some(id),
            Argument::Temporary { id } => Some(id),
            Argument::Immediate { value: _ } => None,
        };
        if let Some(id) = id {
            let v = self.rcs.get_mut(&id).unwrap();
            // decrement the refcount
            let rc = *v - 1;
            //println!("popping {} (rc={})", id, rc);
            if rc == 0 {
                // the register is not in use anymore, remove rc and mark
                // lifetime end
                self.rcs.remove(&id);
                let (_, end) = self.lifetimes.get_mut(&id).unwrap();
                *end = Some(pc as isize);
            } else {
                *v = rc;
            };
        }
        (self, arg)
    }

    fn swap(&mut self, index: usize) -> &mut Self {
        let n = self.args.len() - 1 - 1 - index;
        //println!("n: {}", n);
        // TODO: explain why it's safe to unwrap
        let temp = self.args.get(n).unwrap();
        let temp = temp.clone();
        let top = self.args.len() - 1;
        //println!("top: {}", top);
        self.args[n] = self.args[top];
        self.args[top] = temp;
        self
    }

    fn dup(&mut self, index: usize) -> &mut Self {
        let n = self.args.len() - 1 - index;
        // TODO: explain why it's safe to unwrap
        let arg = self.args.get(n).unwrap();
        let id = match arg {
            Argument::Input { id, address: _ } => Some(id),
            Argument::Temporary { id } => Some(id),
            Argument::Immediate { value: _ } => None,
        };
        if let Some(id) = id {
            let v = self.rcs.get_mut(id).unwrap();
            *v = *v + 1;
        }
        let arg = arg.clone();
        self.args.push(arg);
        self
    }

    // Allocate a new temporary.
    fn alloc_temporary(&mut self) -> (&mut Self, Argument) {
        let arg = Argument::Temporary { id: self.next_id };
        self.next_id += 1;
        (self, arg)
    }

    fn eval_opcode(
        &mut self,
        opcode: EvmOpcode,
        pc: isize,
    ) -> Result<Instr, &str> {
        let (delta, alpha) = opcode.delta_alpha();
        assert!(alpha == 0 || alpha == 1);
        // pop delta arguments off the stack
        let mut args = [ Argument::Immediate { value: U256::default() }; 7];
        let stack = (0..delta).fold(Ok(self), |res, i| {
            if let Ok(stack) = res {
                let (stack, arg) = stack.pop(pc);
                args[i] = arg;
                Ok(stack)
            } else {
                res
            }
        })?;
        // alloc temporary and push it to the stack if alpha == 1
        let (stack, reg) = if alpha > 0 {
            let (stack, reg) = stack.alloc_temporary();
            let stack = stack.push(reg, pc as isize);
            (stack, Some(reg))
        } else {
            (stack, None)
        };
        Ok(Instr::new(opcode, reg, &args[0..delta]))
    }

    fn eval_block<'a>(
        &mut self,
        bytecode: &[u8],
        consts: &mut Vec<U256>,
        instrs: &mut Vec<Instr>
    ) {
        let mut block_pc = 0;
        let mut i = 0;
        while i < bytecode.len() {
            let opcode = unsafe { std::mem::transmute::<_, EvmOpcode>(bytecode[i]) };
            // handle stack opcodes first
            // TODO: use a match expr
            if opcode.is_push() {
                let num_bytes = opcode.push_index() + 1;
                let start = i + 1;
                let end = start + num_bytes;
                let mut buffer: [u8; 32] = [0; 32];
                VmRom::swap_bytes(&bytecode[start..end], &mut buffer);
                let value = U256::from_slice(unsafe { std::mem::transmute::<_, &[u64; 4]>(&buffer) });
                let index = consts.len();
                self.push(Argument::Immediate { value }, block_pc);
                i += num_bytes;
            } else if opcode.is_dup() {
                let index = opcode.dup_index();
                self.dup(index);
            } else if opcode.is_swap() {
                let index = opcode.swap_index();
                self.swap(index);
            } else if opcode == EvmOpcode::POP {
                self.pop(block_pc);
            } else if opcode == EvmOpcode::JUMPDEST {
                // do nothing
                ()
            } else {
                // handle non-stack opcodes
                let res = self.eval_opcode(opcode, block_pc);
                block_pc += 1;
                if let Ok(instr) = res {
                    instrs.push(instr);
                } else {
                    instrs.push(Instr::invalid());
                }
            }
            i += 1;
        }
    }

    fn print_lifetimes(stack: &StaticStack, instr_len: usize) {
        let end_pc = instr_len as isize -1;
        println!("lifetimes: {}", instr_len);
        let mut sorted_lifetimes: Vec<(isize, isize, u16, bool)> = vec!();
        for (k, v) in &stack.lifetimes {
            let id = k;

            let (start, end) = v;
            let end = end.unwrap_or(end_pc);
            let is_input = (*id as usize) < stack.size();
            if is_input {
                println!("i{}: {} to {:?}", id, start, end);
            } else {
                println!("r{}: {} to {:?}", id, start, end);
            }
            sorted_lifetimes.push((*start, end, *id, is_input));
        }
        // sorted by end of life
        sorted_lifetimes.sort_by_key(|v| v.1);
        println!("sorted: {:?}", sorted_lifetimes);
    }

    fn alloc_stack_slots(
        &mut self,
        instrs: &mut [Instr],
        consts: &[U256],
        instr_len: usize,
        block_info: &BbInfo,
    ) {
        // for arg in stack.args.iter() {
        //     println!(">> {:?}", arg);
        // }
        // for instr in instrs.iter() {
        //     println!("{}", Instr::with_consts(instr, &consts));
        // }
        let print_log = false;

        let diff = self.len() as isize - self.size() as isize;
        //println!("diff: {}", diff);

        let mut constraints: HashMap<u16, i16> = HashMap::new();

        let mut ref_address = diff - 1;
        for arg in self.args.iter().rev() {
            //println!("{:?}", arg);
            match arg {
                Argument::Temporary { id } => {
                    //println!("need to allocate @{} to temporary r{}", ref_address, id);
                    constraints.insert(*id, ref_address as i16);
                },
                _ => (),
            }
            ref_address -= 1;
        }
        if self.args.is_empty() {
            //println!("nothing to do because stack at the end is empty");
        }

        //Self::print_lifetimes(self, instr_len);

        let end_pc = instr_len as isize -1;
        //println!("lifetimes:");
        let mut sorted_lifetimes: Vec<(isize, isize, u16, bool, i16)> = vec!();
        for (k, v) in &self.lifetimes {
            let id = k;
            let (start, end) = v;
            let end = end.unwrap_or(end_pc);
            let is_input = (*id as usize) < self.size();
            let addr = if is_input {
                let size = block_info.stack_min_size as i16;
                //println!("size {}", size);
                let addr = (*id as isize - size as isize) as i16;
                //let address = (i as isize - size as isize) as i16;
                addr
            } else {
                std::i16::MAX as i16
            };
            sorted_lifetimes.push((*start, end, *id, is_input, addr));
        }
        // sort by end of life, in case of a tie use start of life
        {
            use std::cmp::Ordering;
            sorted_lifetimes.sort_by(|a, b| {
                match a.1.cmp(&b.1) {
                    Ordering::Equal => a.0.cmp(&b.0),
                    other => other,
                }
            });
        }
        //println!("sorted: {:?}", sorted_lifetimes);

        let mut free_slots: Vec<i16> = vec!();
        for i in 0..block_info.stack_rel_max_size {
            free_slots.push(i as i16);
        }
        let mut pc: isize = 0;
        let mut start_idx = 0;
        while pc < instr_len as isize {
            if print_log { println!("pc: {}", pc) };
            if print_log { println!("free slots: {:?}", free_slots) };
            if print_log { println!("sorted: {:?}", sorted_lifetimes) };
            //let mut max = 0;
            for v in &mut sorted_lifetimes[start_idx..] {
                let (start, end, id, is_input, addr) = *v;
                if pc == end {
                    if print_log { println!("{}{} has reach end of life, its address @{} is available for writing",
                        if is_input { "i" } else { "r" }, id, addr) };
                    assert!(!free_slots.contains(&addr), "@{} is present in free slots", addr);
                    free_slots.push(addr);
                }
                if pc == start {
                    if print_log { println!("{}{} is now alive and need to be allocated to a stack slot!",
                        if is_input { "i" } else { "r" }, id) };
                    if !is_input {
                        let addr = if let Some(addr) = constraints.get(&id) {
                            if print_log { println!("constraining it to @{}", addr) };
                            let idx = free_slots.iter().position(|&x| x == *addr).unwrap();
                            free_slots.remove(idx);
                            *addr
                        } else {
                            // no particular constraint, pick what's free
                            let addr = free_slots.pop().unwrap();
                            if print_log { println!("found free @{}", addr) };
                            addr
                        };
                        v.4 = addr;
                    }
                }
            }
            pc += 1;
        }
        //println!("{:?}", bb);

        // patch instruction temporaries with their allocated stack slots
        for instr in instrs {
            for arg in &mut instr.args {
                match arg {
                    Argument::Temporary { id } => {
                        let res = sorted_lifetimes.iter().find(|&tu| tu.2 == *id);
                        let (_,_,_,_,addr) = res.unwrap();
                        *arg = Argument::Input {
                            id: u16::MAX,
                            address: *addr
                        };
                    },
                    _ => (),
                }
            }
        }
    }

    fn block_fixup(&mut self, instrs: &mut Vec<Instr>) {
        let diff = self.len() as isize - self.size() as isize;
        let mut sets = self.args
            .iter()
            .rev()
            .enumerate()
            .filter_map(|(i, arg)| {
                let ref_address = diff - 1 - i as isize;
                match arg {
                    Argument::Immediate { value } => {
                        Some((Argument::Input { id: u16::MAX, address: ref_address as i16 },
                            Argument::Immediate { value: *value }
                        ))
                    },
                    Argument::Input { id, address } => {
                        if ref_address == *address as isize {
                            // input was unmodified, do nothing
                            None
                        } else  {
                            Some((Argument::Input { id: u16::MAX, address: ref_address as i16 },
                                Argument::Input { id: *id, address: *address }
                            ))
                        }
                    },
                    _ => None,
                }
            });

        loop {
            let s0 = sets.next();
            let s1 = sets.next();
            match (s0, s1) {
                (Some((dst0, src0)), Some((dst1, src1))) => {
                    instrs.push(Instr::set2(dst0, dst1, src0, src1));
                },
                (Some((dst, src)), None) => {
                    instrs.push(Instr::set2(dst, dst, src, src));
                    break;
                },
                (None, None) => break,
                (None, Some(_)) => unreachable!(),
            }
        }

        if diff != 0 {
            // we need to store in last instruction of the block the stack ptr
            // offset
            if let Some(instr) = instrs.last_mut() {
                // check if last instruction has enough bits left, otherwise we
                // need to push a noop jump instruction for that matter
                instr.sp_offset = diff as i16;
            }
        }
    }
}

pub fn build_super_instructions(bytecode: &[u8], schedule: &Schedule) {
    let mut rom = VmRom::new();
    rom.init(&bytecode, &schedule);
    //
    let opcodes: *const EvmOpcode = bytecode.as_ptr() as *const EvmOpcode;
    let mut stack = StaticStack::new();
    let mut consts: Vec<U256> = Vec::new();
    let mut instrs: Vec<Instr> = Vec::new();
    let mut start_instr = 0;

    let block_infos_len = rom.block_infos_len();
    let mut block_offset: isize = 0;
    assert!(block_infos_len > 0);
    for i in 0..block_infos_len {
        println!("\n==== block #{} ====", i);
        let block_info = rom.get_block_info(i);
        let block_bytes_len = if i < (block_infos_len-1) {
            let next_block_info = rom.get_block_info(i+1);
            next_block_info.start_addr.0 - block_info.start_addr.0
        } else {
            bytecode.len() as u16 - block_info.start_addr.0
        } as isize;
        println!("{:?}", block_info);
        println!("block bytes: {}", block_bytes_len);

        // print block opcodes
        let mut offset: isize = 0;
        while offset < block_bytes_len {
            let opcode = unsafe { *opcodes.offset(block_offset + offset) };
            println!("{:?}", opcode);
            if opcode.is_push() {
                let num_bytes = opcode.push_index() as isize + 1;
                offset += num_bytes;
            }
            offset += 1;
        }
        println!("");

        // build super instructions
        stack.clear(block_info.stack_min_size as usize);
        let block = &bytecode[block_offset as usize..(block_offset + block_bytes_len) as usize];
        stack.eval_block(block, &mut consts, &mut instrs);

        let block_instr_len = instrs.len() - start_instr;
        stack.alloc_stack_slots(&mut instrs[start_instr..], &consts, block_instr_len, &block_info);
        stack.block_fixup(&mut instrs);
        start_instr = instrs.len();

        block_offset += block_bytes_len;
    }

    // patch jump addresses

    // compress constants (optional)

    println!("");
    for instr in instrs.iter() {
        let ic = Instr::with_consts(instr, &consts);
        println!("{}", ic);
    }
}
