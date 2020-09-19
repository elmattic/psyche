// Copyright 2020 The Psyche Authors
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

use std::convert::TryFrom;

use crate::instructions::{EvmOpcode, Opcode};
use crate::schedule::Fee::*;
use crate::schedule::{Fee, Schedule};
use crate::u256::*;

#[derive(Debug, PartialEq, Eq)]
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

pub struct VmMemory {
    mmap: Option<memmap::MmapMut>,
    ptr: *mut u8,
    pub len: usize,
}

fn memory_gas_cost(memory_gas: u64, num_words: u64) -> u128 {
    mul_u64(memory_gas, num_words) + mul_u64(num_words, num_words) / 512
}

fn memory_extend_gas_cost(memory_gas: u64, num_words: u64, new_num_words: u64) -> u64 {
    let t0 = mul_u64(num_words, num_words) / 512;
    let t1 = mul_u64(new_num_words, new_num_words) / 512;
    let dt = t1 - t0;
    let d = mul_u64(memory_gas, new_num_words - num_words);
    let delta = dt + d;
    delta.min(u64::max_value() as u128) as u64
}

macro_rules! unsupported_gas {
    () => {
        panic!("unsupported gas amount")
    };
}

impl VmMemory {
    pub fn new() -> VmMemory {
        VmMemory {
            mmap: None,
            ptr: std::ptr::null_mut(),
            len: 0,
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

    pub fn init(&mut self, gas_limit: U256) {
        let max_len = self.find_max_mem_words(gas_limit, &Schedule::default());
        let (num_bytes, overflow) = max_len.overflowing_mul(32);
        if overflow {
            unsupported_gas!();
        }
        let num_bytes = match usize::try_from(num_bytes) {
            Ok(value) => value,
            Err(_) => unsupported_gas!(),
        };
        if num_bytes > 0 {
            match memmap::MmapMut::map_anon(num_bytes) {
                Ok(mut mmap) => {
                    self.ptr = mmap.as_mut_ptr();
                    self.mmap = Some(mmap);
                }
                Err(e) => panic!(e),
            }
        }
    }

    pub fn size(&self) -> usize {
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

    pub fn slice(&self, offset: isize, size: usize) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr.offset(offset), size) }
    }
}

fn num_words(value: u64) -> u64 {
    ((value as u128 + 31) / 32) as u64
}

macro_rules! comment {
    ($lit:literal) => {
        #[cfg(feature = "asm-comment")]
        {
            asm!(concat!("// ", $lit));
        }
    };
}

macro_rules! check_exception_at {
    ($addr:expr, $gas:ident, $rom:ident, $stack:ident, $error:ident) => {
        let bb_info = $rom.get_bb_info($addr);
        let (newgas, oog) = $gas.overflowing_sub(bb_info.gas);
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
    };
}

macro_rules! meter_extend {
    ($new_len:ident, $overflow:ident, $schedule:ident, $memory:ident, $gas:ident, $error:ident) => {
        if !$overflow {
            let len = $memory.len as u64;
            if $new_len > len {
                let cost = memory_extend_gas_cost($schedule.memory_gas, len, $new_len);
                let (newgas, oog) = $gas.overflowing_sub(cost);
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
    };
}

macro_rules! extend_memory {
    ($offset:ident, $size:literal, $schedule:ident, $memory:ident, $gas:ident, $error:ident) => {
        if $offset.le_u64() {
            let (new_len, overflow) = {
                let (temp, overflow) = $offset.low_u64().overflowing_add($size + 31);
                (temp / 32, overflow)
            };
            meter_extend!(new_len, overflow, $schedule, $memory, $gas, $error);
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
            let new_len = if $size.low_u64() == 0 {
                $memory.len as u64
            } else {
                new_len
            };
            meter_extend!(new_len, overflow, $schedule, $memory, $gas, $error);
        } else {
            $error = VmError::OutOfGas;
            break;
        }
    };
}

fn log256(value: u64) -> u64 {
    value.wrapping_sub(1) / 8
}

macro_rules! meter_exp {
    ($exponent_bits:expr, $schedule:ident, $gas:ident, $error:ident) => {
        let fee = $schedule.fees[Fee::ExpByte as usize] as u64;
        let cost = ($exponent_bits > 0) as u64 * fee * (1 + log256($exponent_bits));
        let (newgas, oog) = $gas.overflowing_sub(cost);
        $gas = newgas;
        //if std::intrinsics::unlikely(oog) {
        if oog {
            $error = VmError::OutOfGas;
            break;
        }
    };
}

macro_rules! meter_sha3 {
    ($size:ident, $schedule:ident, $gas:ident, $error:ident) => {
        let fee = $schedule.fees[Fee::Sha3Word as usize] as u64;
        let (cost, ovf) = num_words($size.low_u64()).overflowing_mul(fee);
        let (newgas, oog) = $gas.overflowing_sub(cost as u64);
        $gas = newgas;
        if oog | ovf | !$size.le_u64() {
            $error = VmError::OutOfGas;
            break;
        }
    };
}

#[derive(Debug)]
pub struct ReturnData {
    pub offset: usize,
    pub size: usize,
    pub gas: u64,
    pub error: VmError,
}

impl ReturnData {
    pub fn new(offset: usize, size: usize, gas: u64, error: VmError) -> Self {
        ReturnData {
            offset,
            size,
            gas,
            error,
        }
    }

    pub fn ok(offset: usize, size: usize, gas: u64) -> Self {
        ReturnData {
            offset,
            size,
            gas,
            error: VmError::None,
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
            let gas = $gas;
            let ssize = $stack.size();
            let msize = $memory.size();
            $hook($pc, gas, ssize, msize);
        }
    };
}

pub unsafe fn run_evm(
    bytecode: &[u8],
    rom: &VmRom,
    schedule: &Schedule,
    gas_limit: U256,
    memory: &mut VmMemory,
) -> ReturnData {
    // TODO: use MaybeUninit
    let mut slots: VmStackSlots = std::mem::uninitialized();
    let mut stack: VmStack = VmStack::new(&mut slots);
    let code: *const Opcode = rom.code() as *const Opcode;
    let mut pc: usize = 0;
    let mut gas = gas_limit.low_u64();
    let mut error: VmError = VmError::None;
    let mut entered = false;
    while !entered {
        entered = true;
        check_exception_at!(0, gas, rom, stack, error);
        return ReturnData::new(0, 0, gas, error);
    }
    loop {
        let opcode = *code.offset(pc as isize);
        lldb_hook!(pc, gas, stack, memory, lldb_hook_single_step);
        //println!("{:?}", opcode);
        match opcode {
            Opcode::STOP => {
                lldb_hook!(pc, gas, stack, memory, lldb_hook_stop);
                break;
            }
            Opcode::ADD => {
                comment!("opADD");
                let a = stack.pop_u256();
                let b = stack.pop_u256();
                let result = add_u256(a, b);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::MUL => {
                comment!("opMUL");
                let a = stack.pop_u256();
                let b = stack.pop_u256();
                let result = mul_u256(a, b);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::SUB => {
                comment!("opSUB");
                let a = stack.pop_u256();
                let b = stack.pop_u256();
                let result = sub_u256(a, b);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::DIV => {
                comment!("opDIV");
                let a = stack.pop_u256();
                let b = stack.pop_u256();
                let q = div_u256(a, b);
                stack.push(q);
                //
                pc += 1;
            }
            Opcode::MOD => {
                comment!("opMOD");
                let a = stack.pop_u256();
                let b = stack.pop_u256();
                let r = mod_u256(a, b);
                stack.push(r);
                //
                pc += 1;
            }
            Opcode::SDIV => {
                comment!("opSDIV");
                let a = stack.pop_u256();
                let b = stack.pop_u256();
                let r = sdiv_u256(a, b);
                stack.push(r);
                //
                pc += 1;
            }
            Opcode::SMOD => {
                comment!("opSMOD");
                let a = stack.pop_u256();
                let b = stack.pop_u256();
                let r = smod_u256(a, b);
                stack.push(r);
                //
                pc += 1;
            }
            Opcode::ADDMOD => {
                comment!("opADDMOD");
                let a = stack.pop_u256();
                let b = stack.pop_u256();
                let c = stack.pop_u256();
                let r = addmod_u256(a, b, c);
                stack.push(r);
                //
                pc += 1;
            }
            Opcode::MULMOD => {
                comment!("opMULMOD");
                let a = stack.pop_u256();
                let b = stack.pop_u256();
                let c = stack.pop_u256();
                let r = mulmod_u256(a, b, c);
                stack.push(r);
                //
                pc += 1;
            }
            Opcode::EXP => {
                comment!("opEXP");
                let a = stack.pop_u256();
                let b = stack.pop_u256();
                let exponent_bits = 256 - leading_zeros_u256(b);
                meter_exp!(exponent_bits as u64, schedule, gas, error);
                let result = exp_u256(a, b, exponent_bits);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::SIGNEXTEND => {
                comment!("opSIGNEXTEND");
                let a = stack.pop();
                let b = stack.pop();
                let result = signextend_u256(a, b);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::LT => {
                comment!("opLT");
                let a = stack.pop_u256();
                let b = stack.pop_u256();
                let result = U256::from_u64(lt_u256(a, b) as u64);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::GT => {
                comment!("opGT");
                let a = stack.pop_u256();
                let b = stack.pop_u256();
                let result = U256::from_u64(gt_u256(a, b) as u64);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::SLT => {
                comment!("opSLT");
                let a = stack.pop_u256();
                let b = stack.pop_u256();
                let result = U256::from_u64(slt_u256(a, b) as u64);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::SGT => {
                comment!("opSGT");
                let a = stack.pop_u256();
                let b = stack.pop_u256();
                let result = U256::from_u64(sgt_u256(a, b) as u64);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::EQ => {
                comment!("opEQ");
                let a = stack.pop();
                let b = stack.pop();
                let result = eq_u256(a, b);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::ISZERO => {
                comment!("opISZERO");
                let a = stack.pop();
                let result = iszero_u256(a);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::AND => {
                comment!("opAND");
                let a = stack.pop();
                let b = stack.pop();
                let result = and_u256(a, b);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::OR => {
                comment!("opOR");
                let a = stack.pop();
                let b = stack.pop();
                let result = or_u256(a, b);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::XOR => {
                comment!("opXOR");
                let a = stack.pop();
                let b = stack.pop();
                let result = xor_u256(a, b);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::NOT => {
                comment!("opNOT");
                let a = stack.pop();
                let result = not_u256(a);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::BYTE => {
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
            Opcode::SHL => {
                comment!("opSHL");
                let a = stack.pop();
                let b = stack.pop();
                let result = shl_u256(a, b);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::SHR => {
                comment!("opSHR");
                let a = stack.pop();
                let b = stack.pop();
                let result = shr_u256(a, b, false);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::SAR => {
                comment!("opSAR");
                let a = stack.pop();
                let b = stack.pop();
                let result = shr_u256(a, b, true);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::SHA3 => {
                comment!("opSHA3");
                let offset = stack.pop_u256();
                let size = stack.pop_u256();
                meter_sha3!(size, schedule, gas, error);
                extend_memory!(offset, size, schedule, memory, gas, error);
                let offset = offset.low_u64() as isize;
                let size = size.low_u64() as usize;
                let result = sha3_u256(memory.ptr.offset(offset), size);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::ADDRESS
            | Opcode::BALANCE
            | Opcode::ORIGIN
            | Opcode::CALLER
            | Opcode::CALLVALUE
            | Opcode::CALLDATALOAD
            | Opcode::CALLDATASIZE
            | Opcode::CALLDATACOPY => unimplemented!(),
            Opcode::CODESIZE => {
                comment!("opCODESIZE");
                stack.push(U256::from_u64(bytecode.len() as u64));
                //
                pc += 1;
            }
            Opcode::CODECOPY
            | Opcode::GASPRICE
            | Opcode::EXTCODESIZE
            | Opcode::EXTCODECOPY
            | Opcode::RETURNDATASIZE
            | Opcode::RETURNDATACOPY
            | Opcode::EXTCODEHASH
            | Opcode::BLOCKHASH
            | Opcode::COINBASE
            | Opcode::TIMESTAMP
            | Opcode::NUMBER
            | Opcode::DIFFICULTY
            | Opcode::GASLIMIT
            | Opcode::CHAINID
            | Opcode::SELFBALANCE => unimplemented!(),
            Opcode::POP => {
                comment!("opPOP");
                stack.pop();
                //
                pc += 1;
            }
            Opcode::MLOAD => {
                comment!("opMLOAD");
                let offset = stack.pop_u256();
                extend_memory!(offset, 32, schedule, memory, gas, error);
                let result = memory.read(offset.low_u64() as usize);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::MSTORE => {
                comment!("opMSTORE");
                let offset = stack.pop_u256();
                let value = stack.pop();
                extend_memory!(offset, 32, schedule, memory, gas, error);
                memory.write(offset.low_u64() as usize, value);
                //
                pc += 1;
            }
            Opcode::MSTORE8 => {
                comment!("opMSTORE8");
                let offset = stack.pop_u256();
                let value = stack.pop().low_u64();
                extend_memory!(offset, 1, schedule, memory, gas, error);
                memory.write_byte(offset.low_u64() as usize, value as u8);
                //
                pc += 1;
            }
            Opcode::SLOAD | Opcode::SSTORE => unimplemented!(),
            Opcode::JUMP => {
                comment!("opJUMP");
                let addr = stack.pop();
                let in_bounds = is_ltpow2_u256(addr, VmRom::MAX_CODESIZE);
                let low = addr.low_u64();
                if in_bounds & rom.is_jumpdest(low) {
                    pc = low as usize + 1;
                    check_exception_at!(low, gas, rom, stack, error);
                    break;
                } else {
                    error = VmError::InvalidJumpDest;
                    break;
                }
            }
            Opcode::JUMPI => {
                comment!("opJUMPI");
                let addr = stack.pop();
                let cond = stack.pop();
                if is_zero_u256(cond) {
                    pc += 1;
                    check_exception_at!(pc as u64, gas, rom, stack, error);
                    break;
                } else {
                    let in_bounds = is_ltpow2_u256(addr, VmRom::MAX_CODESIZE);
                    let low = addr.low_u64();
                    if in_bounds & rom.is_jumpdest(low) {
                        pc = low as usize + 1;
                        check_exception_at!(low, gas, rom, stack, error);
                        break;
                    } else {
                        error = VmError::InvalidJumpDest;
                        break;
                    }
                }
            }
            Opcode::PC => {
                comment!("opPC");
                let result = U256::from_u64(pc as u64);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::MSIZE => {
                comment!("opMSIZE");
                let result = U256::from_u64(memory.size() as u64);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::GAS => {
                comment!("opGAS");
                let result = U256::from_u64(gas);
                stack.push(result);
                //
                pc += 1;
                check_exception_at!(pc as u64, gas, rom, stack, error);
                break;
            }
            Opcode::JUMPDEST => {
                comment!("opJUMPDEST");
                //
                pc += 1;
            }
            Opcode::BEGINSUB | Opcode::RETURNSUB | Opcode::JUMPSUB => unimplemented!(),
            Opcode::PUSH1 => {
                comment!("opPUSH1");
                let result = *(code.offset(pc as isize + 1) as *const u8);
                let result = U256::from_u64(result as u64);
                stack.push(result);
                //
                pc += 2;
            }
            Opcode::PUSH2 => {
                comment!("opPUSH2");
                let result = *(code.offset(pc as isize + 1) as *const u16);
                let result = U256::from_u64(result as u64);
                stack.push(result);
                //
                pc += 3;
            }
            Opcode::PUSH4 => {
                comment!("opPUSH4");
                let result = *(code.offset(pc as isize + 1) as *const u32);
                let result = U256::from_u64(result as u64);
                stack.push(result);
                //
                pc += 5;
            }
            Opcode::PUSH3
            | Opcode::PUSH5
            | Opcode::PUSH6
            | Opcode::PUSH7
            | Opcode::PUSH8
            | Opcode::PUSH9
            | Opcode::PUSH10
            | Opcode::PUSH11
            | Opcode::PUSH12
            | Opcode::PUSH13
            | Opcode::PUSH14
            | Opcode::PUSH15
            | Opcode::PUSH16 => {
                comment!("opPUSH16");
                let num_bytes = (opcode.push_index() as i32) + 1;
                let result = load16_u256(code.offset(pc as isize + 1) as *const U256, num_bytes);
                stack.push(result);
                //
                pc += 1 + num_bytes as usize;
            }
            Opcode::PUSH17
            | Opcode::PUSH18
            | Opcode::PUSH19
            | Opcode::PUSH20
            | Opcode::PUSH21
            | Opcode::PUSH22
            | Opcode::PUSH23
            | Opcode::PUSH24
            | Opcode::PUSH25
            | Opcode::PUSH26
            | Opcode::PUSH27
            | Opcode::PUSH28
            | Opcode::PUSH29
            | Opcode::PUSH30
            | Opcode::PUSH31
            | Opcode::PUSH32 => {
                comment!("opPUSH32");
                let num_bytes = (opcode.push_index() as i32) + 1;
                let result = load32_u256(code.offset(pc as isize + 1) as *const U256, num_bytes);
                stack.push(result);
                //
                pc += 1 + num_bytes as usize;
            }
            Opcode::DUP1 => {
                comment!("opDUP1");
                let result = stack.peek();
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::DUP2 => {
                comment!("opDUP2");
                let result = stack.peek1();
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::DUP3
            | Opcode::DUP4
            | Opcode::DUP5
            | Opcode::DUP6
            | Opcode::DUP7
            | Opcode::DUP8
            | Opcode::DUP9
            | Opcode::DUP10
            | Opcode::DUP11
            | Opcode::DUP12
            | Opcode::DUP13
            | Opcode::DUP14
            | Opcode::DUP15
            | Opcode::DUP16 => {
                comment!("opDUPn");
                let index = opcode.dup_index();
                let result = stack.peekn(index);
                stack.push(result);
                //
                pc += 1;
            }
            Opcode::SWAP1 => {
                comment!("opSWAP1");
                let a = stack.pop();
                let b = stack.pop();
                stack.push(a);
                stack.push(b);
                //
                pc += 1;
            }
            Opcode::SWAP2 => {
                comment!("opSWAP2");
                let value = stack.peek();
                let prev = stack.set(2, value);
                stack.pop();
                stack.push(prev);
                //
                pc += 1;
            }
            Opcode::SWAP3
            | Opcode::SWAP4
            | Opcode::SWAP5
            | Opcode::SWAP6
            | Opcode::SWAP7
            | Opcode::SWAP8
            | Opcode::SWAP9
            | Opcode::SWAP10
            | Opcode::SWAP11
            | Opcode::SWAP12
            | Opcode::SWAP13
            | Opcode::SWAP14
            | Opcode::SWAP15
            | Opcode::SWAP16 => {
                comment!("opSWAPn");
                let value = stack.peek();
                let index = opcode.swap_index();
                let prev = stack.set(index, value);
                stack.pop();
                stack.push(prev);
                //
                pc += 1;
            }
            Opcode::LOG0
            | Opcode::LOG1
            | Opcode::LOG2
            | Opcode::LOG3
            | Opcode::LOG4
            | Opcode::CREATE
            | Opcode::CALL
            | Opcode::CALLCODE => unimplemented!(),
            Opcode::RETURN => {
                lldb_hook!(pc, gas, stack, memory, lldb_hook_stop);
                comment!("opRETURN");
                let offset = stack.pop_u256();
                let size = stack.pop_u256();
                extend_memory!(offset, size, schedule, memory, gas, error);
                return ReturnData::ok(offset.low_u64() as usize, size.low_u64() as usize, gas);
            }
            Opcode::DELEGATECALL | Opcode::CREATE2 | Opcode::STATICCALL | Opcode::REVERT => {
                unimplemented!()
            }
            Opcode::INVALID => {
                error = VmError::InvalidInstruction;
                break;
            }
            Opcode::SELFDESTRUCT => unimplemented!(),
        }
    }
    return ReturnData::new(0, 0, gas, error);
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
        } else {
            0
        };
        BbInfo {
            stack_min_size,
            stack_rel_max_size: stack_rel_max_size,
            gas,
        }
    }
}

pub struct VmRom {
    data: [u8; VmRom::SIZE],
}

impl VmRom {
    /// EIP-170 states a max contract code size of 2**14 + 2**13, we round it
    /// to the next power of two.
    const MAX_CODESIZE: usize = 32768;
    const JUMPDESTS_SIZE: usize = VmRom::MAX_CODESIZE / 8;
    const BB_INFOS_SIZE: usize = VmRom::MAX_CODESIZE * std::mem::size_of::<BbInfo>();
    const SIZE: usize = VmRom::MAX_CODESIZE + VmRom::JUMPDESTS_SIZE + VmRom::BB_INFOS_SIZE;
    const BB_INFOS_OFFSET: usize = VmRom::MAX_CODESIZE + VmRom::JUMPDESTS_SIZE;

    pub fn new() -> VmRom {
        VmRom {
            data: [0; VmRom::SIZE],
        }
    }

    fn code(&self) -> *const u8 {
        self.data.as_ptr()
    }

    fn is_jumpdest(&self, addr: u64) -> bool {
        let jump_dests =
            unsafe { self.data.as_ptr().offset(VmRom::MAX_CODESIZE as isize) as *mut u64 };
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

    fn swap_bytes(input: &[u8], swapped: &mut [u8]) {
        for i in 0..input.len() {
            swapped[input.len() - 1 - i] = input[i];
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
            fn basic(
                addr: u32,
                stack_min_size: u16,
                stack_max_size: u16,
                stack_end_size: u16,
                gas: u64,
            ) -> BlockInfo {
                BlockInfo {
                    addr,
                    stack_min_size,
                    stack_max_size,
                    stack_end_size,
                    gas,
                    is_basic_block: true,
                }
            }
            fn partial(
                addr: u32,
                stack_min_size: u16,
                stack_max_size: u16,
                stack_end_size: u16,
                gas: u64,
            ) -> BlockInfo {
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
        const OPCODE_INFOS: [(Fee, u16, u16); 256] = [
            (Zero, 0, 0), /* STOP = 0x00 */
            (VeryLow, 2, 1), /* ADD = 0x01 */
            (Low, 2, 1), /* MUL = 0x02 */
            (VeryLow, 2, 1), /* SUB = 0x03 */
            (Low, 2, 1), /* DIV = 0x04 */
            (Low, 2, 1), /* SDIV = 0x05 */
            (Low, 2, 1), /* MOD = 0x06 */
            (Low, 2, 1), /* SMOD = 0x07 */
            (Mid, 3, 1), /* ADDMOD = 0x08 */
            (Mid, 3, 1), /* MULMOD = 0x09 */
            (Exp, 2, 1), /* EXP = 0x0a */
            (Low, 2, 1), /* SIGNEXTEND = 0x0b */
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (VeryLow, 2, 1), /* LT = 0x10 */
            (VeryLow, 2, 1), /* GT = 0x11 */
            (VeryLow, 2, 1), /* SLT = 0x12 */
            (VeryLow, 2, 1), /* SGT = 0x13 */
            (VeryLow, 2, 1), /* EQ = 0x14 */
            (VeryLow, 1, 1), /* ISZERO = 0x15 */
            (VeryLow, 2, 1), /* AND = 0x16 */
            (VeryLow, 2, 1), /* OR = 0x17 */
            (VeryLow, 2, 1), /* XOR = 0x18 */
            (VeryLow, 1, 1), /* NOT = 0x19 */
            (VeryLow, 2, 1), /* BYTE = 0x1a */
            (VeryLow, 2, 1), /* SHL = 0x1b */
            (VeryLow, 2, 1), /* SHR = 0x1c */
            (VeryLow, 2, 1), /* SAR = 0x1d */
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Sha3, 2, 1), /* SHA3 = 0x20 */
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Base, 0, 1), /* ADDRESS = 0x30 */
            (Balance, 1, 1), /* BALANCE = 0x31 */
            (Base, 0, 1), /* ORIGIN = 0x32 */
            (Base, 0, 1), /* CALLER = 0x33 */
            (Base, 0, 1), /* CALLVALUE = 0x34 */
            (VeryLow, 1, 1), /* CALLDATALOAD = 0x35 */
            (Base, 0, 1), /* CALLDATASIZE = 0x36 */
            (Copy, 3, 0), /* CALLDATACOPY = 0x37 */
            (Base, 0, 1), /* CODESIZE = 0x38 */
            (Copy, 3, 0), /* CODECOPY = 0x39 */
            (Base, 0, 1), /* GASPRICE = 0x3a */
            (Zero, 1, 1), /* EXTCODESIZE = 0x3b */
            (Zero, 4, 0), /* EXTCODECOPY = 0x3c */
            (Base, 0, 1), /* RETURNDATASIZE = 0x3d */
            (Copy, 3, 0), /* RETURNDATACOPY = 0x3e */
            (Zero, 1, 1), /* EXTCODEHASH = 0x3f */
            (Blockhash, 1, 1), /* BLOCKHASH = 0x40 */
            (Base, 0, 1), /* COINBASE = 0x41 */
            (Base, 0, 1), /* TIMESTAMP = 0x42 */
            (Base, 0, 1), /* NUMBER = 0x43 */
            (Base, 0, 1), /* DIFFICULTY = 0x44 */
            (Base, 0, 1), /* GASLIMIT = 0x45 */
            (Base, 0, 1), /* CHAINID = 0x46 */
            (Low, 0, 1), /* SELFBALANCE = 0x47 */
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Base, 1, 0), /* POP = 0x50 */
            (VeryLow, 1, 1), /* MLOAD = 0x51 */
            (VeryLow, 2, 0), /* MSTORE = 0x52 */
            (VeryLow, 2, 0), /* MSTORE8 = 0x53 */
            (Zero, 1, 1), /* SLOAD = 0x54 */
            (Zero, 2, 0), /* SSTORE = 0x55 */
            (Mid, 1, 0), /* JUMP = 0x56 */
            (High, 2, 0), /* JUMPI = 0x57 */
            (Base, 0, 1), /* PC = 0x58 */
            (Base, 0, 1), /* MSIZE = 0x59 */
            (Base, 0, 1), /* GAS = 0x5a */
            (Jumpdest, 0, 0), /* JUMPDEST = 0x5b */
            (Base, 0, 0), /* BEGINSUB = 0x5c */
            (Low, 0, 0), /* RETURNSUB = 0x5d */
            (High, 1, 0), /* JUMPSUB = 0x5e */
            (Zero, 0, 0),
            (VeryLow, 0, 1), /* PUSH1 = 0x60 */
            (VeryLow, 0, 1), /* PUSH2 = 0x61 */
            (VeryLow, 0, 1), /* PUSH3 = 0x62 */
            (VeryLow, 0, 1), /* PUSH4 = 0x63 */
            (VeryLow, 0, 1), /* PUSH5 = 0x64 */
            (VeryLow, 0, 1), /* PUSH6 = 0x65 */
            (VeryLow, 0, 1), /* PUSH7 = 0x66 */
            (VeryLow, 0, 1), /* PUSH8 = 0x67 */
            (VeryLow, 0, 1), /* PUSH9 = 0x68 */
            (VeryLow, 0, 1), /* PUSH10 = 0x69 */
            (VeryLow, 0, 1), /* PUSH11 = 0x6a */
            (VeryLow, 0, 1), /* PUSH12 = 0x6b */
            (VeryLow, 0, 1), /* PUSH13 = 0x6c */
            (VeryLow, 0, 1), /* PUSH14 = 0x6d */
            (VeryLow, 0, 1), /* PUSH15 = 0x6e */
            (VeryLow, 0, 1), /* PUSH16 = 0x6f */
            (VeryLow, 0, 1), /* PUSH17 = 0x70 */
            (VeryLow, 0, 1), /* PUSH18 = 0x71 */
            (VeryLow, 0, 1), /* PUSH19 = 0x72 */
            (VeryLow, 0, 1), /* PUSH20 = 0x73 */
            (VeryLow, 0, 1), /* PUSH21 = 0x74 */
            (VeryLow, 0, 1), /* PUSH22 = 0x75 */
            (VeryLow, 0, 1), /* PUSH23 = 0x76 */
            (VeryLow, 0, 1), /* PUSH24 = 0x77 */
            (VeryLow, 0, 1), /* PUSH25 = 0x78 */
            (VeryLow, 0, 1), /* PUSH26 = 0x79 */
            (VeryLow, 0, 1), /* PUSH27 = 0x7a */
            (VeryLow, 0, 1), /* PUSH28 = 0x7b */
            (VeryLow, 0, 1), /* PUSH29 = 0x7c */
            (VeryLow, 0, 1), /* PUSH30 = 0x7d */
            (VeryLow, 0, 1), /* PUSH31 = 0x7e */
            (VeryLow, 0, 1), /* PUSH32 = 0x7f */
            (VeryLow, 1, 2), /* DUP1 = 0x80 */
            (VeryLow, 2, 3), /* DUP2 = 0x81 */
            (VeryLow, 3, 4), /* DUP3 = 0x82 */
            (VeryLow, 4, 5), /* DUP4 = 0x83 */
            (VeryLow, 5, 6), /* DUP5 = 0x84 */
            (VeryLow, 6, 7), /* DUP6 = 0x85 */
            (VeryLow, 7, 8), /* DUP7 = 0x86 */
            (VeryLow, 8, 9), /* DUP8 = 0x87 */
            (VeryLow, 9, 10), /* DUP9 = 0x88 */
            (VeryLow, 10, 11), /* DUP10 = 0x89 */
            (VeryLow, 11, 12), /* DUP11 = 0x8a */
            (VeryLow, 12, 13), /* DUP12 = 0x8b */
            (VeryLow, 13, 14), /* DUP13 = 0x8c */
            (VeryLow, 14, 15), /* DUP14 = 0x8d */
            (VeryLow, 15, 16), /* DUP15 = 0x8e */
            (VeryLow, 16, 17), /* DUP16 = 0x8f */
            (VeryLow, 2, 2), /* SWAP1 = 0x90 */
            (VeryLow, 3, 3), /* SWAP2 = 0x91 */
            (VeryLow, 4, 4), /* SWAP3 = 0x92 */
            (VeryLow, 5, 5), /* SWAP4 = 0x93 */
            (VeryLow, 6, 6), /* SWAP5 = 0x94 */
            (VeryLow, 7, 7), /* SWAP6 = 0x95 */
            (VeryLow, 8, 8), /* SWAP7 = 0x96 */
            (VeryLow, 9, 9), /* SWAP8 = 0x97 */
            (VeryLow, 10, 10), /* SWAP9 = 0x98 */
            (VeryLow, 11, 11), /* SWAP10 = 0x99 */
            (VeryLow, 12, 12), /* SWAP11 = 0x9a */
            (VeryLow, 13, 13), /* SWAP12 = 0x9b */
            (VeryLow, 14, 14), /* SWAP13 = 0x9c */
            (VeryLow, 15, 15), /* SWAP14 = 0x9d */
            (VeryLow, 16, 16), /* SWAP15 = 0x9e */
            (VeryLow, 17, 17), /* SWAP16 = 0x9f */
            (Zero, 2, 0), /* LOG0 = 0xa0 */
            (Zero, 3, 0), /* LOG1 = 0xa1 */
            (Zero, 4, 0), /* LOG2 = 0xa2 */
            (Zero, 5, 0), /* LOG3 = 0xa3 */
            (Zero, 6, 0), /* LOG4 = 0xa4 */
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 3, 1), /* CREATE = 0xf0 */
            (Zero, 7, 1), /* CALL = 0xf1 */
            (Zero, 7, 1), /* CALLCODE = 0xf2 */
            (Zero, 2, 0), /* RETURN = 0xf3 */
            (Zero, 6, 1), /* DELEGATECALL = 0xf4 */
            (Zero, 4, 1), /* CREATE2 = 0xf5 */
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 6, 1), /* STATICCALL = 0xfa */
            (Zero, 0, 0),
            (Zero, 0, 0),
            (Zero, 2, 0), /* REVERT = 0xfd */
            (Zero, 0, 0), /* INVALID = 0xfe */
            (Zero, 1, 0), /* SELFDESTRUCT = 0xff */
        ];
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
            let opcode = unsafe { std::mem::transmute::<u8, EvmOpcode>(code) };
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
            } else {
                i += 1;
            }
            if opcode.is_terminator() || i >= bytecode.len() {
                block_infos.push(BlockInfo::basic(
                    addr,
                    stack_min_size,
                    stack_max_size,
                    stack_size,
                    gas,
                ));
                addr = i as u32;
                stack_size = 0;
                stack_min_size = 0;
                stack_max_size = 0;
                gas = 0;
            } else {
                let code = bytecode[i];
                let opcode = unsafe { std::mem::transmute::<u8, EvmOpcode>(code) };
                if opcode == EvmOpcode::JUMPDEST {
                    block_infos.push(BlockInfo::partial(
                        addr,
                        stack_min_size,
                        stack_max_size,
                        stack_size,
                        gas,
                    ));
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
            } else {
                let (more, needed) = if stack_min_size > info.stack_end_size {
                    (0, (stack_min_size - info.stack_end_size))
                } else {
                    // case info.stack_end_size >= stack_min_size
                    (info.stack_end_size - stack_min_size, 0)
                };
                stack_min_size = info.stack_min_size.saturating_add(needed);
                stack_max_size = max(
                    info.stack_max_size.saturating_add(needed),
                    stack_max_size.saturating_add(more),
                );
                gas += info.gas;
            }
            unsafe {
                let bb_info = BbInfo::new(stack_min_size, stack_max_size, gas);
                *bb_infos.offset(info.addr as isize) = bb_info;
            }
        }
    }

    pub fn init(&mut self, bytecode: &[u8], schedule: &Schedule) {
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
                let opcode = unsafe { std::mem::transmute::<u8, EvmOpcode>(code) };
                self.data[i] = opcode.to_internal() as u8;
                if opcode.is_push() {
                    let num_bytes = opcode.push_index() + 1;
                    let start = i + 1;
                    let end = start + num_bytes;
                    let dest = &mut self.data[start..end];
                    VmRom::swap_bytes(&bytecode[start..end], dest);
                    i += 1 + num_bytes;
                } else {
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
        let jump_dests = unsafe { self.data.as_mut_ptr().offset(jump_dests_offset) as *mut u64 };
        let mut bits: u64 = 0;
        let mut i: usize = 0;
        while i < bytecode.len() {
            // save i for later in j
            let j = i;
            let code = bytecode[i];
            let opcode = unsafe { std::mem::transmute::<u8, EvmOpcode>(code) };
            if opcode.is_push() {
                let num_bytes = opcode.push_index() + 1;
                i += 1 + num_bytes;
            } else {
                if opcode == EvmOpcode::JUMPDEST {
                    bits |= 1u64 << (i % 64);
                }
                i += 1;
            }
            let do_write = (j % 64) > (i % 64);
            if do_write {
                let offset = (j / 64) as isize;
                unsafe { *jump_dests.offset(offset) = bits }
                bits = 0;
            }
        }
        let offset = (i / 64) as isize;
        unsafe { *jump_dests.offset(offset) = bits }
        //
        self.write_bb_infos(bytecode, schedule);
    }
}
