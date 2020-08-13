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

extern crate clap;
#[macro_use]
extern crate num_derive;

mod assembler;
mod instructions;
mod schedule;
mod utils;
mod u256;
mod vm;

use clap::{Arg, App, SubCommand};

use std::convert::TryFrom;
use std::fmt::{self, Write};
use std::fs;

use instructions::{EvmOpcode, EvmInstruction};
use schedule::{Schedule};
use utils::{encode_hex, decode_hex, print_config};
use u256::U256;
use vm::{VmMemory, VmRom, run_evm};

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

fn evm(bytes: &Vec<u8>, gas_limit: U256) {
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
}

fn asm(filename: &str) {
    let code = fs::read_to_string(filename).expect("Something went wrong reading the file");
    let result = assembler::from_string(&code);
    match result {
        Ok(v) => println!("{}", encode_hex(&v)),
        Err(e) => println!("{:?}", e),
    }
}

fn kick(filename: &str) {
    let code = fs::read_to_string(filename).expect("Something went wrong reading the file");
    let result = assembler::from_string(&code);
    match result {
        Ok(bytes) => evm(&bytes, U256::from_u64(VM_DEFAULT_GAS)),
        Err(e) => println!("{:?}", e),
    }
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
            .subcommand(SubCommand::with_name("asm")
                .about("Assemble EVM bytecode")
                .arg(Arg::with_name("INPUT")
                    .index(1)
                    .required(true)
                    .help("The .ass file to assemble")))
            .subcommand(SubCommand::with_name("kick")
                .about("Assemble EVM bytecode and run it")
                .arg(Arg::with_name("INPUT")
                    .index(1)
                    .required(true)
                    .help("The .ass file to assemble and run")))
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
    if let Some(matches) = matches.subcommand_matches("evm") {
        let mut gas = U256::from_u64(VM_DEFAULT_GAS);
        if let Some(value) = matches.value_of("GAS") {
            match U256::from_dec_str(value) {
                Ok(temp) => gas = temp,
                Err(err) => println!("Invalid --gas: {:?}", err)
            }
        }
        let hex_str = matches.value_of("CODE").unwrap();
        match decode_hex(hex_str) {
            Ok(bytes) => evm(&bytes, gas),
            Err(e) => println!("{:?}", e)
        }
        return;
    }
    if let Some(matches) = matches.subcommand_matches("asm") {
        let filename = matches.value_of("INPUT").unwrap();
        asm(filename);
        return;
    }
    if let Some(matches) = matches.subcommand_matches("kick") {
        let filename = matches.value_of("INPUT").unwrap();
        kick(filename);
        return;
    }
    if let Some(matches) = matches.subcommand_matches("disasm") {
        let code = matches.value_of("CODE").unwrap();
        disasm(code);
        return;
    }
}
