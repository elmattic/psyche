// Copyright 2019 The Psyche Authors
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

//#![feature(asm)]

extern crate clap;
extern crate ethereum_types;
extern crate uint;
extern crate memmap;
#[macro_use]
extern crate num_derive;
extern crate num_traits;

mod instructions;
mod schedule;
mod u256;

use clap::{Arg, App, SubCommand};
use std::convert::TryFrom;
use std::fmt;
use std::{fmt::Write, num::ParseIntError};
use instructions::{EvmOpcode, EvmInstruction, Opcode};
use instructions::Opcode::*;
use schedule::{Fee, Schedule};
use schedule::Fee::*;
use u256::*;

#[derive(Debug)]
pub enum VmError {
    None,
    StackUnderflow,
    StackOverflow,
    OutOfGas,
    InvalidJumpDest,
    InvalidInstruction,
}

struct VmStackSlots([U256; VmStack::LEN]);

struct VmStack {
    start: *const U256,
    sp: *mut U256,
}

impl VmStack {
    pub const LEN: usize = 1024;

    pub unsafe fn new(slots: &mut VmStackSlots) -> VmStack {
        VmStack {
            start: slots.0.as_ptr(),
            // sp is always pointing at the top of the stack
            sp: slots.0.as_mut_ptr().offset(-1),
        }
    }

    pub unsafe fn push(&mut self, value: U256) {
        self.sp = self.sp.offset(1);
        store_u256(self.sp, value, 0);
    }

    pub unsafe fn pop(&mut self) -> U256 {
        let temp = self.peek();
        self.sp = self.sp.offset(-1);
        temp
    }

    pub unsafe fn pop_u256(&mut self) -> U256 {
        let temp = *self.sp;
        self.sp = self.sp.offset(-1);
        temp
    }

    pub unsafe fn peek(&self) -> U256 {
        self.peekn(0)
    }

    pub unsafe fn peek1(&self) -> U256 {
        self.peekn(1)
    }

    pub unsafe fn peekn(&self, index: usize) -> U256 {
        load_u256(self.sp, -(index as isize))
    }

    pub unsafe fn set(&self, index: usize, value: U256) -> U256 {
        let offset = -(index as isize);
        let temp = load_u256(self.sp, offset);
        store_u256(self.sp, value, offset);
        temp
    }

    pub unsafe fn size(&self) -> usize {
        const WORD_SIZE: usize = std::mem::size_of::<U256>();
        usize::wrapping_sub(self.sp.offset(1) as _, self.start as _) / WORD_SIZE
    }
}

struct VmMemory {
    mmap: Option<memmap::MmapMut>,
    ptr: *mut u8,
    pub len: usize
}

fn memory_gas_cost(memory_gas: u64, num_words: u64) -> u128 {
    mul_u64(memory_gas, num_words) + mul_u64(num_words, num_words) / 512
}

fn memory_extend_gas_cost(memory_gas: u64, num_words: u64, new_num_words: u64) -> u128 {
    let t0 = mul_u64(num_words, num_words) / 512;
    let t1 = mul_u64(new_num_words, new_num_words) / 512;
    let dt = t1 - t0;
    let d = mul_u64(memory_gas, (new_num_words - num_words));
    let delta = dt + d;
    delta
}

macro_rules! unsupported_gas {
    () => {
        panic!("unsupported gas amount")
    }
}

impl VmMemory {
    fn new() -> VmMemory {
        VmMemory {
            mmap: None,
            ptr: std::ptr::null_mut(),
            len: 0
        }
    }

    fn find_max_mem_words(&self, gas_limit: U256, sched: &Schedule) -> u64 {
        if (gas_limit.0[2] > 0) | (gas_limit.0[3] > 0) {
            unsupported_gas!();
        }
        let gas_limit = gas_limit.low_u128();
        let mut l: u64 = 0;
        let mut r: u64 = u64::max_value();
        let mut result: u64 = 0;
        while l < r {
            let mid = l + (r - l) / 2;
            let cost: u128 = memory_gas_cost(sched.memory_gas, mid);
            if cost > gas_limit {
                r = mid;
            } else {
                l = mid + 1;
                result = mid;
            }
        }
        result
    }

    fn init(&mut self, gas_limit: U256) {
        let max_len = self.find_max_mem_words(gas_limit, &Schedule::default());
        let (num_bytes, overflow) = max_len.overflowing_mul(32);
        if overflow {
            unsupported_gas!();
        }
        let num_bytes = match usize::try_from(num_bytes) {
            Ok(value) => value,
            Err(_) => unsupported_gas!()
        };
        if num_bytes > 0 {
            match memmap::MmapMut::map_anon(num_bytes) {
                Ok(mut mmap) => {
                    self.ptr = mmap.as_mut_ptr();
                    self.mmap = Some(mmap);
                }
                Err(e) => panic!(e)
            }
        }
    }

    fn size(&self) -> usize {
        self.len * std::mem::size_of::<U256>()
    }

    unsafe fn read(&mut self, offset: usize) -> U256 {
        let src = self.ptr.offset(offset as isize);
        let result = bswap_u256(loadu_u256(src as *const U256, 0));
        return result;
    }

    unsafe fn write(&mut self, offset: usize, value: U256) {
        let dest = self.ptr.offset(offset as isize);
        storeu_u256(dest as *mut U256, bswap_u256(value), 0);
    }

    unsafe fn write_byte(&mut self, offset: usize, value: u8) {
        let dest = self.ptr.offset(offset as isize);
        *dest = value;
    }

    unsafe fn slice(&self, offset: isize, size: usize) -> &[u8] {
        std::slice::from_raw_parts(self.ptr.offset(offset), size)
    }
}

macro_rules! comment {
   ($lit:literal) => (
        #[cfg(feature = "asm-comment")]
        {
            asm!(concat!("// ", $lit));
        }
    )
}

macro_rules! check_exception_at {
    ($addr:expr, $gas:ident, $rom:ident, $stack:ident, $error:ident) => {
        let bb_info = $rom.get_bb_info($addr);
        let (newgas, oog) = overflowing_sub_word($gas, bb_info.gas);
        $gas = newgas;
        let stack_min_size = bb_info.stack_min_size as usize;
        let stack_rel_max_size = bb_info.stack_rel_max_size as usize;
        let stack_size = $stack.size();
        let underflow = stack_size < stack_min_size;
        let overflow = (stack_size + stack_rel_max_size) > VmStack::LEN;
        if !(oog | underflow | overflow) {
            continue;
        }
        if oog {
            $error = VmError::OutOfGas;
        }
        if underflow {
            $error = VmError::StackUnderflow;
        }
        if overflow {
            $error = VmError::StackOverflow;
        }
    }
}

macro_rules! metered_extend {
    ($new_len:ident, $overflow:ident, $schedule:ident, $memory:ident, $gas:ident, $error:ident) => {
        if !$overflow {
            let len = $memory.len as u64;
            if $new_len > len {
                let cost = memory_extend_gas_cost($schedule.memory_gas, len, $new_len);
                let (newgas, oog) = overflowing_sub_word_u128($gas, cost);
                $gas = newgas;
                if !oog {
                    $memory.len = $new_len as usize;
                } else {
                    $error = VmError::OutOfGas;
                    break;
                }
            }
        } else {
            $error = VmError::OutOfGas;
            break;
        }
    }
}

macro_rules! extend_memory {
    ($offset:ident, $size:literal, $schedule:ident, $memory:ident, $gas:ident, $error:ident) => {
        if $offset.le_u64() {
            let (new_len, overflow) = {
                let (temp, overflow) = $offset.low_u64().overflowing_add($size + 31);
                (temp / 32, overflow)
            };
            metered_extend!(new_len, overflow, $schedule, $memory, $gas, $error);
        } else {
            $error = VmError::OutOfGas;
            break;
        }
    };
    ($offset:ident, $size:ident, $schedule:ident, $memory:ident, $gas:ident, $error:ident) => {
        if $offset.le_u64() & $size.le_u64() {
            let (new_len, overflow) = {
                let (temp1, overflow1) = $offset.low_u64().overflowing_add($size.low_u64());
                let (temp2, overflow2) = temp1.overflowing_add(31);
                (temp2 / 32, overflow1 | overflow2)
            };
            metered_extend!(new_len, overflow, $schedule, $memory, $gas, $error);
        } else {
            $error = VmError::OutOfGas;
            break;
        }
    }
}

#[derive(Debug)]
pub struct ReturnData {
    offset: usize,
    size: usize,
    gas: u64
}

impl ReturnData {
    pub fn new(offset: usize, size: usize, gas: u64) -> Self {
        ReturnData {
            offset: offset,
            size: size,
            gas: gas
        }
    }
}

fn lldb_hook_single_step(pc: usize, gas: u64, ssize: usize, msize: usize) {}
fn lldb_hook_stop(pc: usize, gas: u64, ssize: usize, msize: usize) {}

macro_rules! lldb_hook {
    ($pc:expr, $gas:expr, $stack:ident, $memory:ident, $hook:ident) => {
        #[cfg(debug_assertions)]
        {
            let stack_start = $stack.start;
            let gas = $gas.as_u256().low_u64();
            let ssize = $stack.size();
            let msize = $memory.size();
            $hook($pc, gas, ssize, msize);
        }
    }
}

unsafe fn run_evm(bytecode: &[u8], rom: &VmRom, schedule: &Schedule, gas_limit: U256, memory: &mut VmMemory) -> ReturnData {
    // TODO: use MaybeUninit
    let mut slots: VmStackSlots = std::mem::uninitialized();
    let mut stack: VmStack = VmStack::new(&mut slots);
    let code: *const Opcode = rom.code() as *const Opcode;
    let mut pc: usize = 0;
    let mut gas: Word = Word::from_slice(&(gas_limit.0));
    let mut error: VmError = VmError::None;
    let mut entered = false;
    while !entered {
        entered = true;
        check_exception_at!(0, gas, rom, stack, error);
        panic!("{:?}", error);
    }
    loop {
        let opcode = *code.offset(pc as isize);
        lldb_hook!(pc, gas, stack, memory, lldb_hook_single_step);
        //println!("{:?}", opcode);
        match opcode {
            STOP => {
                lldb_hook!(pc, gas, stack, memory, lldb_hook_stop);
                break;
            },
            ADD => {
                comment!("opADD");
                let a = stack.pop_u256();
                let b = stack.pop_u256();
                let result = add_u256(a, b);
                stack.push(result);
                //
                pc += 1;
            }
            MUL => {
                comment!("opMUL");
                let a = stack.pop_u256();
                let b = stack.pop_u256();
                let result = mul_u256(a, b);
                stack.push(result);
                //
                pc += 1;
            }
            SUB => {
                comment!("opSUB");
                let a = stack.pop_u256();
                let b = stack.pop_u256();
                let result = sub_u256(a, b);
                stack.push(result);
                //
                pc += 1;
            }
            DIV | MOD | SDIV | SMOD | ADDMOD | MULMOD | EXP => unimplemented!(),
            SIGNEXTEND => {
                comment!("opSIGNEXTEND");
                let offset = *(stack.sp as *const u32) % 32;
                let offset = offset as isize;
                let value = *((stack.sp.offset(-1) as *const u8).offset(offset));
                let a = stack.pop();
                let b = stack.pop();
                let result = signextend_u256(a, b, value);
                stack.push(result);
                //
                pc += 1;
            }
            LT => unimplemented!(),
            GT => {
                comment!("opGT");
                let a = stack.pop_u256();
                let b = stack.pop_u256();
                let result = U256::from_u64(gt_u256(a, b) as u64);
                stack.push(result);
                //
                pc += 1;
            }
            SLT | SGT => unimplemented!(),
            EQ => {
                comment!("opEQ");
                let a = stack.pop();
                let b = stack.pop();
                let result = eq_u256(a, b);
                stack.push(result);
                //
                pc += 1;
            }
            ISZERO => {
                comment!("opISZERO");
                let a = stack.pop();
                let result = iszero_u256(a);
                stack.push(result);
                //
                pc += 1;
            }
            AND => {
                comment!("opAND");
                let a = stack.pop();
                let b = stack.pop();
                let result = and_u256(a, b);
                stack.push(result);
                //
                pc += 1;
            }
            OR => {
                comment!("opOR");
                let a = stack.pop();
                let b = stack.pop();
                let result = or_u256(a, b);
                stack.push(result);
                //
                pc += 1;
            }
            XOR => {
                comment!("opXOR");
                let a = stack.pop();
                let b = stack.pop();
                let result = xor_u256(a, b);
                stack.push(result);
                //
                pc += 1;
            }
            NOT => {
                comment!("opNOT");
                let a = stack.pop();
                let result = not_u256(a);
                stack.push(result);
                //
                pc += 1;
            }
            BYTE => {
                comment!("opBYTE");
                let a = stack.peek();
                let lt32 = is_ltpow2_u256(a, 32);
                let offset = 31 - (a.0[0] % 32);
                let offset = offset as isize;
                let value = *((stack.sp.offset(-1) as *const u8).offset(offset));
                let value = value as u64;
                let result = U256::from_u64((lt32 as u64) * value);
                stack.pop();
                stack.pop();
                stack.push(result);
                //
                pc += 1;
            }
            SHL => {
                comment!("opSHL");
                let a = stack.pop();
                let b = stack.pop();
                let result = shl_u256(a, b);
                stack.push(result);
                //
                pc += 1;
            }
            SHR | SAR => unimplemented!(),
            SHA3 | ADDRESS | BALANCE | ORIGIN | CALLER | CALLVALUE | CALLDATALOAD | CALLDATASIZE | CALLDATACOPY => unimplemented!(),
            CODESIZE => {
                comment!("opCODESIZE");
                stack.push(U256::from_u64(bytecode.len() as u64));
                //
                pc += 1;
            }
            CODECOPY | GASPRICE | EXTCODESIZE | EXTCODECOPY | RETURNDATASIZE | RETURNDATACOPY | EXTCODEHASH | BLOCKHASH | COINBASE | TIMESTAMP | NUMBER | DIFFICULTY | GASLIMIT | CHAINID | SELFBALANCE => unimplemented!(),
            POP => {
                comment!("opPOP");
                stack.pop();
                //
                pc += 1;
            }
            MLOAD => {
                comment!("opMLOAD");
                let offset = stack.pop_u256();
                extend_memory!(offset, 32, schedule, memory, gas, error);
                let result = memory.read(offset.low_u64() as usize);
                stack.push(result);
                //
                pc += 1;
            },
            MSTORE => {
                comment!("opMSTORE");
                let offset = stack.pop_u256();
                let value = stack.pop();
                extend_memory!(offset, 32, schedule, memory, gas, error);
                memory.write(offset.low_u64() as usize, value);
                //
                pc += 1;
            },
            MSTORE8 => {
                comment!("opMSTORE8");
                let offset = stack.pop_u256();
                let value = stack.pop().low_u64();
                extend_memory!(offset, 1, schedule, memory, gas, error);
                memory.write_byte(offset.low_u64() as usize, value as u8);
                //
                pc += 1;
            },
            SLOAD | SSTORE => unimplemented!(),
            JUMP => {
                comment!("opJUMP");
                let addr = stack.pop();
                let in_bounds = is_ltpow2_u256(addr, VmRom::MAX_CODESIZE);
                let low = addr.low_u64();
                if in_bounds & rom.is_jumpdest(low) {
                    pc = low as usize + 1;
                    check_exception_at!(low, gas, rom, stack, error);
                    break;
                }
                else {
                    error = VmError::InvalidJumpDest;
                    break;
                }
            }
            JUMPI => {
                comment!("opJUMPI");
                let addr = stack.pop();
                let cond = stack.pop();
                if is_zero_u256(cond) {
                    pc += 1;
                    check_exception_at!(pc as u64, gas, rom, stack, error);
                    break;
                }
                else {
                    let in_bounds = is_ltpow2_u256(addr, VmRom::MAX_CODESIZE);
                    let low = addr.low_u64();
                    if in_bounds & rom.is_jumpdest(low) {
                        pc = low as usize + 1;
                        check_exception_at!(low, gas, rom, stack, error);
                        break;
                    }
                    else {
                        error = VmError::InvalidJumpDest;
                        break;
                    }
                }
            }
            PC => {
                comment!("opPC");
                let result = U256::from_u64(pc as u64);
                stack.push(result);
                //
                pc += 1;
            }
            MSIZE => {
                comment!("opMSIZE");
                let result = U256::from_u64(memory.size() as u64);
                stack.push(result);
                //
                pc += 1;
            }
            GAS => {
                comment!("opGAS");
                let result = gas.as_u256();
                stack.push(result);
                //
                pc += 1;
                check_exception_at!(pc as u64, gas, rom, stack, error);
                break;
            }
            JUMPDEST => {
                comment!("opJUMPDEST");
                //
                pc += 1;
            }
            BEGINSUB | RETURNSUB | JUMPSUB => unimplemented!(),
            PUSH1 => {
                comment!("opPUSH1");
                let result = *(code.offset(pc as isize + 1) as *const u8);
                let result = U256::from_u64(result as u64);
                stack.push(result);
                //
                pc += 2;
            }
            PUSH2 => {
                comment!("opPUSH2");
                let result = *(code.offset(pc as isize + 1) as *const u16);
                let result = U256::from_u64(result as u64);
                stack.push(result);
                //
                pc += 3;
            }
            PUSH4 => {
                comment!("opPUSH4");
                let result = *(code.offset(pc as isize + 1) as *const u32);
                let result = U256::from_u64(result as u64);
                stack.push(result);
                //
                pc += 5;
            }
            PUSH3 | PUSH5 | PUSH6 | PUSH7 | PUSH8 | PUSH9 | PUSH10 | PUSH11 |
            PUSH12 | PUSH13 | PUSH14 | PUSH15 | PUSH16 => {
                comment!("opPUSH16");
                let num_bytes = (opcode.push_index() as i32) + 1;
                let result = load16_u256(code.offset(pc as isize + 1) as *const U256, num_bytes);
                stack.push(result);
                //
                pc += 1 + num_bytes as usize;
            }
            PUSH17 | PUSH18 | PUSH19 | PUSH20 | PUSH21 | PUSH22 | PUSH23 |
            PUSH24 | PUSH25 | PUSH26 | PUSH27 | PUSH28 | PUSH29 | PUSH30 |
            PUSH31 | PUSH32 => {
                comment!("opPUSH32");
                let num_bytes = (opcode.push_index() as i32) + 1;
                let result = load32_u256(code.offset(pc as isize + 1) as *const U256, num_bytes);
                stack.push(result);
                //
                pc += 1 + num_bytes as usize;
            }
            DUP1 => {
                comment!("opDUP1");
                let result = stack.peek();
                stack.push(result);
                //
                pc += 1;
            }
            DUP2 => {
                comment!("opDUP2");
                let result = stack.peek1();
                stack.push(result);
                //
                pc += 1;
            }
            DUP3 | DUP4 | DUP5 | DUP6 | DUP7 | DUP8 | DUP9 | DUP10 | DUP11 |
            DUP12 | DUP13 | DUP14 | DUP15 | DUP16 => {
                comment!("opDUPn");
                let index = opcode.dup_index();
                let result = stack.peekn(index);
                stack.push(result);
                //
                pc += 1;
            }
            SWAP1 => {
                comment!("opSWAP1");
                let a = stack.pop();
                let b = stack.pop();
                stack.push(a);
                stack.push(b);
                //
                pc += 1;
            }
            SWAP2 => {
                comment!("opSWAP2");
                let value = stack.peek();
                let prev = stack.set(2, value);
                stack.pop();
                stack.push(prev);
                //
                pc += 1;
            }
            SWAP3 | SWAP4 | SWAP5 | SWAP6 | SWAP7 | SWAP8 | SWAP9 | SWAP10 |
            SWAP11 | SWAP12 | SWAP13 | SWAP14 | SWAP15 | SWAP16 => {
                comment!("opSWAPn");
                let value = stack.peek();
                let index = opcode.swap_index();
                let prev = stack.set(index, value);
                stack.pop();
                stack.push(prev);
                //
                pc += 1;
            }
            LOG0 | LOG1 | LOG2 | LOG3 | LOG4 | CREATE | CALL | CALLCODE => unimplemented!(),
            RETURN => {
                lldb_hook!(pc, gas, stack, memory, lldb_hook_stop);
                comment!("opRETURN");
                let offset = stack.pop_u256();
                let size = stack.pop_u256();
                extend_memory!(offset, size, schedule, memory, gas, error);
                return ReturnData::new(offset.low_u64() as usize, size.low_u64() as usize, 0)
            }
            DELEGATECALL | CREATE2 | STATICCALL | REVERT => unimplemented!(),
            INVALID => {
                error = VmError::InvalidInstruction;
                break;
            }
            SELFDESTRUCT => unimplemented!(),
        }
    }
    if let VmError::None = error {
        return ReturnData::new(0, 0, 0);
    }
    panic!("{:?}", error);
}

fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| {
            let temp: Result<u8, _> = u8::from_str_radix(&s[i..i + 2], 16);
            match temp {
                Ok(_) => temp,
                Err(e) => Err(e)
            }
        })
        .collect()
}

#[derive(Debug)]
struct BbInfo {
    stack_min_size: u16,
    stack_rel_max_size: u16,
    gas: u64,
}
impl BbInfo {
    fn new(stack_min_size: u16, stack_max_size: u16, gas: u64) -> BbInfo {
        let stack_rel_max_size = if stack_max_size > stack_min_size {
            stack_max_size - stack_min_size
        }
        else {
            0
        };
        BbInfo {
            stack_min_size,
            stack_rel_max_size: stack_rel_max_size,
            gas,
        }
    }
}

struct VmRom {
    data: [u8; VmRom::SIZE]
}

impl VmRom {
    /// EIP-170 states a max contract code size of 2**14 + 2**13, we round it
    /// to the next power of two.
    const MAX_CODESIZE: usize = 32768;

    const JUMPDESTS_SIZE: usize = VmRom::MAX_CODESIZE / 8;

    const BB_INFOS_SIZE: usize = VmRom::MAX_CODESIZE * std::mem::size_of::<BbInfo>();

    const SIZE: usize = VmRom::MAX_CODESIZE + VmRom::JUMPDESTS_SIZE + VmRom::BB_INFOS_SIZE;

    const BB_INFOS_OFFSET: usize = VmRom::MAX_CODESIZE + VmRom::JUMPDESTS_SIZE;

    fn new() -> VmRom {
        VmRom { data: [0; VmRom::SIZE] }
    }

    fn code(&self) -> *const u8 {
        self.data.as_ptr()
    }

    fn is_jumpdest(&self, addr: u64) -> bool {
        let jump_dests = unsafe {
            self.data.as_ptr().offset(VmRom::MAX_CODESIZE as isize) as *mut u64
        };
        let offset = (addr % (VmRom::MAX_CODESIZE as u64)) as isize;
        let bits = unsafe { *jump_dests.offset(offset / 64) };
        let mask = 1u64 << (offset % 64);
        (bits & mask) != 0
    }

    fn get_bb_info(&self, addr: u64) -> &BbInfo {
        unsafe {
            let offset = VmRom::BB_INFOS_OFFSET as isize;
            let bb_infos = self.data.as_ptr().offset(offset) as *mut BbInfo;
            &*bb_infos.offset(addr as isize)
        }
    }

    fn swap_bytes(input: &[u8], swapped: &mut[u8]) {
        for i in 0..input.len() {
            swapped[input.len()-1-i] = input[i];
        }
    }

    fn write_bb_infos(&mut self, bytecode: &[u8], schedule: &Schedule) {
        use std::cmp::max;
        #[derive(Debug)]
        struct BlockInfo {
            addr: u32,
            stack_min_size: u16,
            stack_max_size: u16,
            stack_end_size: u16,
            gas: u64,
            is_basic_block: bool,
        }
        impl BlockInfo {
            fn basic(addr: u32, stack_min_size: u16, stack_max_size: u16, stack_end_size: u16, gas: u64) -> BlockInfo {
                BlockInfo {
                    addr,
                    stack_min_size,
                    stack_max_size,
                    stack_end_size,
                    gas,
                    is_basic_block: true,
                }
            }
            fn partial(addr: u32, stack_min_size: u16, stack_max_size: u16, stack_end_size: u16, gas: u64) -> BlockInfo {
                BlockInfo {
                    addr,
                    stack_min_size,
                    stack_max_size,
                    stack_end_size,
                    gas,
                    is_basic_block: false,
                }
            }
        }
        const OPCODE_INFOS: [(Fee, u16, u16); 256] = [(Zero, 0, 0), (VeryLow, 2, 1), (Low, 2, 1), (VeryLow, 2, 1), (Low, 2, 1), (Low, 2, 1), (Low, 2, 1), (Low, 2, 1), (Mid, 3, 1), (Mid, 3, 1), (Exp, 2, 1), (Low, 2, 1), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (VeryLow, 2, 1), (VeryLow, 2, 1), (VeryLow, 2, 1), (VeryLow, 2, 1), (VeryLow, 2, 1), (VeryLow, 1, 1), (VeryLow, 2, 1), (VeryLow, 2, 1), (VeryLow, 2, 1), (VeryLow, 1, 1), (VeryLow, 2, 1), (VeryLow, 2, 1), (VeryLow, 2, 1), (VeryLow, 2, 1), (Zero, 0, 0), (Zero, 0, 0), (Sha3, 2, 1), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Base, 0, 1), (Balance, 1, 1), (Base, 0, 1), (Base, 0, 1), (Base, 0, 1), (VeryLow, 1, 1), (Base, 0, 1), (Copy, 3, 0), (Base, 0, 1), (Copy, 3, 0), (Base, 0, 1), (Zero, 1, 1), (Zero, 4, 0), (Base, 0, 1), (Copy, 3, 0), (Zero, 1, 1), (Blockhash, 1, 1), (Base, 0, 1), (Base, 0, 1), (Base, 0, 1), (Base, 0, 1), (Base, 0, 1), (Base, 0, 1), (Low, 0, 1), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Base, 1, 0), (VeryLow, 1, 1), (VeryLow, 2, 0), (VeryLow, 2, 0), (Zero, 1, 1), (Zero, 2, 0), (Mid, 1, 0), (High, 2, 0), (Base, 0, 1), (Base, 0, 1), (Base, 0, 1), (Jumpdest, 0, 0), (Base, 0, 0), (Low, 0, 0), (High, 1, 0), (Zero, 0, 0), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 0, 1), (VeryLow, 1, 2), (VeryLow, 2, 3), (VeryLow, 3, 4), (VeryLow, 4, 5), (VeryLow, 5, 6), (VeryLow, 6, 7), (VeryLow, 7, 8), (VeryLow, 8, 9), (VeryLow, 9, 10), (VeryLow, 10, 11), (VeryLow, 11, 12), (VeryLow, 12, 13), (VeryLow, 13, 14), (VeryLow, 14, 15), (VeryLow, 15, 16), (VeryLow, 16, 17), (VeryLow, 2, 2), (VeryLow, 3, 3), (VeryLow, 4, 4), (VeryLow, 5, 5), (VeryLow, 6, 6), (VeryLow, 7, 7), (VeryLow, 8, 8), (VeryLow, 9, 9), (VeryLow, 10, 10), (VeryLow, 11, 11), (VeryLow, 12, 12), (VeryLow, 13, 13), (VeryLow, 14, 14), (VeryLow, 15, 15), (VeryLow, 16, 16), (VeryLow, 17, 17), (Zero, 2, 0), (Zero, 3, 0), (Zero, 4, 0), (Zero, 5, 0), (Zero, 6, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 3, 1), (Zero, 7, 1), (Zero, 7, 1), (Zero, 2, 0), (Zero, 6, 1), (Zero, 4, 1), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 0, 0), (Zero, 6, 1), (Zero, 0, 0), (Zero, 0, 0), (Zero, 2, 0), (Zero, 0, 0), (Zero, 1, 0)];
        let mut addr: u32 = 0;
        let mut stack_size: u16 = 0;
        let mut stack_min_size: u16 = 0;
        let mut stack_max_size: u16 = 0;
        let mut gas: u64 = 0;
        let mut block_infos: Vec<BlockInfo> = Vec::with_capacity(1024);
        // forward pass over the bytecode
        let mut i: usize = 0;
        while i < bytecode.len() {
            let code = bytecode[i];
            let opcode = unsafe {
                std::mem::transmute::<u8, EvmOpcode>(code)
            };
            let (fee, delta, alpha) = OPCODE_INFOS[code as usize];
            // new_stack_size is (stack_size + needed + alpha) - delta
            // and represents the new stack size after the opcode has been
            // dispatched
            let (new_stack_size, needed) = if delta > stack_size {
                    (alpha, (delta - stack_size))
                } else {
                    // case stack_size >= delta
                    ((stack_size - delta).saturating_add(alpha), 0)
                };
            stack_size = new_stack_size;
            stack_min_size = stack_min_size.saturating_add(needed);
            stack_max_size = max(stack_max_size, new_stack_size);
            // TODO: overflow possible?
            gas += fee.gas(schedule) as u64;
            if opcode.is_push() {
                let num_bytes = opcode.push_index() + 1;
                i += 1 + num_bytes;
            }
            else {
                i += 1;
            }
            if opcode.is_terminator() || i >= bytecode.len() {
                block_infos.push(BlockInfo::basic(
                    addr, stack_min_size, stack_max_size, stack_size, gas)
                );
                addr = i as u32;
                stack_size = 0;
                stack_min_size = 0;
                stack_max_size = 0;
                gas = 0;
            }
            else {
                let code = bytecode[i];
                let opcode = unsafe {
                    std::mem::transmute::<u8, EvmOpcode>(code)
                };
                if opcode == EvmOpcode::JUMPDEST {
                    block_infos.push(BlockInfo::partial(
                        addr, stack_min_size, stack_max_size, stack_size, gas)
                    );
                    addr = i as u32;
                    stack_size = 0;
                    stack_min_size = 0;
                    stack_max_size = 0;
                    gas = 0;
                }
            }
        }
        // backward pass, write BB infos to rom
        let bb_infos = unsafe {
            let offset = VmRom::BB_INFOS_OFFSET as isize;
            self.data.as_ptr().offset(offset) as *mut BbInfo
        };
        for info in block_infos.iter().rev() {
            if info.is_basic_block {
                stack_min_size = info.stack_min_size;
                stack_max_size = info.stack_max_size;
                gas = info.gas;
            }
            else {
                let (more, needed) = if stack_min_size > info.stack_end_size {
                    (0, (stack_min_size - info.stack_end_size))
                } else {
                    // case info.stack_end_size >= stack_min_size
                    (info.stack_end_size - stack_min_size, 0)
                };
                stack_min_size = info.stack_min_size.saturating_add(needed);
                stack_max_size = max(
                    info.stack_max_size.saturating_add(needed),
                    stack_max_size.saturating_add(more)
                );
                gas += info.gas;
            }
            unsafe {
                let bb_info = BbInfo::new(stack_min_size, stack_max_size, gas);
                *bb_infos.offset(info.addr as isize) = bb_info;
            }
        }
    }

    fn init(&mut self, bytecode: &[u8], schedule: &Schedule) {
        // erase rom
        for b in &mut self.data[..] {
            *b = 0;
        }
        if bytecode.len() > VmRom::MAX_CODESIZE {
            panic!("bytecode is too big ({:?} bytes)", bytecode.len());
        }
        // copy bytecode
        #[cfg(target_endian = "little")]
        {
            // reverse `PUSHN` opcode bytes
            let mut i: usize = 0;
            while i < bytecode.len() {
                let code = bytecode[i];
                let opcode = unsafe {
                    std::mem::transmute::<u8, EvmOpcode>(code)
                };
                self.data[i] = opcode.to_internal() as u8;
                if opcode.is_push() {
                    let num_bytes = opcode.push_index() + 1;
                    let start = i + 1;
                    let end = start + num_bytes;
                    let dest = &mut self.data[start..end];
                    VmRom::swap_bytes(&bytecode[start..end], dest);
                    i += 1 + num_bytes;
                }
                else {
                    i += 1;
                }
            }
        }
        #[cfg(target_endian = "big")]
        {
            unimplemented!();
        }
        // write valid jump destinations
        let jump_dests_offset = VmRom::MAX_CODESIZE as isize;
        let jump_dests = unsafe {
            self.data.as_mut_ptr().offset(jump_dests_offset) as *mut u64
        };
        let mut bits: u64 = 0;
        let mut i: usize = 0;
        while i < bytecode.len() {
            // save i for later in j
            let j = i;
            let code = bytecode[i];
            let opcode = unsafe {
                std::mem::transmute::<u8, EvmOpcode>(code)
            };
            if opcode.is_push() {
                let num_bytes = opcode.push_index() + 1;
                i += 1 + num_bytes;
            }
            else {
                if opcode == EvmOpcode::JUMPDEST {
                    bits |= 1u64 << (i % 64);
                }
                i += 1;
            }
            let do_write = (j % 64) > (i % 64);
            if do_write {
                let offset = (j / 64) as isize;
                unsafe {
                    *jump_dests.offset(offset) = bits
                }
                bits = 0;
            }
        }
        let offset = (i / 64) as isize;
        unsafe {
            *jump_dests.offset(offset) = bits
        }
        //
        self.write_bb_infos(bytecode, schedule);
    }
}

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut temp = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        let _ = write!(&mut temp, "{:02x}", b);
    }
    temp
}

macro_rules! test_feature_bit {
   ($name:ident, $register: ident, $mask:expr) => (
        fn $name() -> bool {
            use core::arch::x86_64::__cpuid;
            #[cfg(target_arch = "x86_64")]
            {
                let result = unsafe { __cpuid(1) };
                return (result.$register & $mask) > 0;
            }
            return false;
        }
    )
}

macro_rules! test_extented_feature_bit {
   ($name:ident, $register: ident, $mask:expr) => (
        fn $name() -> bool {
            use core::arch::x86_64::__cpuid_count;
            #[cfg(target_arch = "x86_64")]
            {
                let result = unsafe { __cpuid_count(7, 0) };
                return (result.$register & $mask) > 0;
            }
            return false;
        }
    )
}

test_feature_bit!(may_i_use_Ssse3, ecx, 1 << 9);
test_extented_feature_bit!(may_i_use_Avx2, ebx, 1 << 5);
test_extented_feature_bit!(may_i_use_Bmi1, ebx, 1 << 3);
test_extented_feature_bit!(may_i_use_Bmi2, ebx, 1 << 8);
test_extented_feature_bit!(may_i_use_Adx, ebx, 1 << 19);
test_extented_feature_bit!(may_i_use_Avx512f, ebx, 1 << 16);

#[allow(unreachable_code)]
fn print_config() {
    #[cfg(debug_assertions)]
    {
        println!("mode: debug");
    }
    #[cfg(not(debug_assertions))]
    {
        println!("mode: release");
    }
    let mut features = vec![];
    if may_i_use_Ssse3() { features.push("ssse3"); }
    if may_i_use_Avx2() { features.push("avx2"); }
    if may_i_use_Bmi1() { features.push("bmi1"); }
    if may_i_use_Bmi2() { features.push("bmi2"); }
    if may_i_use_Adx() { features.push("adx"); }
    if may_i_use_Avx512f() { features.push("avx512f"); }
    if features.len() > 0 {
        println!("flags: {}", features.join(" "));
    }
}

const VM_DEFAULT_GAS: u64 = 20_000_000_000_000;

struct Bytecode<'a> {
    data: &'a [u8],
    addr: usize
}

impl<'a> Bytecode<'a> {
    fn new(bytes: &'a [u8]) -> Bytecode<'a> {
        Bytecode { data: bytes, addr: 0 }
    }
}

struct IncompletePushError {
    addr: usize
}

impl fmt::Display for IncompletePushError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "incomplete push instruction at 0x{:04x}", self.addr)
    }
}

impl<'a> Iterator for Bytecode<'a> {
    type Item = Result<EvmInstruction<'a>, IncompletePushError>;
    fn next(&mut self) -> Option<Result<EvmInstruction<'a>, IncompletePushError>> {
        if self.addr < self.data.len() {
            let value = self.data[self.addr];
            match EvmOpcode::try_from(value) {
                Ok(opcode) => {
                    if opcode.is_push() {
                        let num_bytes = opcode.push_index() + 1;
                        let start = self.addr + 1;
                        let end = start + num_bytes;
                        if (end-1) < self.data.len() {
                            let temp = EvmInstruction::MultiByte {
                                addr: self.addr,
                                opcode: opcode,
                                bytes: &self.data[start..end]
                            };
                            self.addr += 1 + num_bytes;
                            Some(Ok(temp))
                        } else {
                            Some(Err(IncompletePushError { addr: self.addr }))
                        }
                    }
                    else {
                        let temp = EvmInstruction::SingleByte {
                            addr: self.addr,
                            opcode: opcode
                        };
                        self.addr += 1;
                        Some(Ok(temp))
                    }
                },
                Err(_) => {
                    let temp = EvmInstruction::SingleByte {
                        addr: self.addr,
                        opcode: EvmOpcode::INVALID
                    };
                    self.addr += 1;
                    Some(Ok(temp))
                }
            }
        } else {
            None
        }
    }
}

fn disasm(input: &str) {
    let temp = decode_hex(input);
    match temp {
        Ok(bytes) => {
            let result: Result<Vec<EvmInstruction>, _> = Bytecode::new(&bytes).collect();
            match result {
                Ok(x) => {
                    let asm = x
                        .iter()
                        .map(|i| match i {
                            EvmInstruction::SingleByte { addr, opcode } => {
                                format!("{:04x}:    {}", addr, opcode)
                            },
                            EvmInstruction::MultiByte { addr, opcode, bytes } => {
                                let imm = encode_hex(bytes);
                                format!("{:04x}:    {} 0x{}", addr, opcode, imm)
                            },
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    println!("{}", asm);
                },
                Err(e) => println!("{}", e)
            }
        }
        Err(e) => println!("{:?}", e)
    }
}

fn evm(input: &str, gas_limit: U256) {
    let temp = decode_hex(input);
    match temp {
        Ok(bytes) => {
            //println!("{} bytes", bytes.len());
            let schedule = Schedule::default();
            let mut rom = VmRom::new();
            rom.init(&bytes, &schedule);
            let mut memory = VmMemory::new();
            memory.init(gas_limit);
            let slice = unsafe {
                let ret_data = run_evm(&bytes, &rom, &schedule, gas_limit, &mut memory);
                memory.slice(ret_data.offset as isize, ret_data.size)
            };
            let mut buffer = String::with_capacity(512);
            for byte in slice {
                let _ = write!(buffer, "{:02x}", byte);
            }
            println!("0x{:}", buffer);
        },
        Err(e) => println!("{:?}", e)
    };
}

fn main() {
    let matches =
        App::new("Psyche")
            .subcommand(SubCommand::with_name("evm")
                .about("Run EVM bytecode")
                .arg(Arg::with_name("CODE")
                    .index(1)
                    .required(true)
                    .help("Contract code as hex (without 0x)"))
                .arg(Arg::with_name("GAS")
                    .takes_value(true)
                    .short("g")
                    .long("gas")
                    .help("Supplied gas as decimal")))
            .subcommand(SubCommand::with_name("disasm")
                .about("Disassemble EVM bytecode")
                .arg(Arg::with_name("CODE")
                    .index(1)
                    .required(true)
                    .help("Contract code as hex (without 0x)")))
            .arg(Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Sets verbose output"))
            .get_matches();

    if matches.is_present("verbose") {
        print_config();
    }
    if let Some(matches) = matches.subcommand_matches("disasm") {
        let code = matches.value_of("CODE").unwrap();
        disasm(code);
        return;
    }
    if let Some(matches) = matches.subcommand_matches("evm") {
        let mut gas = U256::from_u64(VM_DEFAULT_GAS);
        if let Some(value) = matches.value_of("GAS") {
            match U256::from_dec_str(value) {
                Ok(temp) => gas = temp,
                Err(err) => println!("Invalid --gas: {:?}", err)
            }
        }
        let code = matches.value_of("CODE").unwrap();
        evm(code, gas);
        return;
    }
}
