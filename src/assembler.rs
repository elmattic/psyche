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

use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, take_while1};
use nom::character::complete::{
    alpha1, alphanumeric1, char, digit1, oct_digit1, hex_digit1, multispace0,
    newline, not_line_ending,
};
use nom::combinator::{map, not, opt, recognize, value};
use nom::multi::{many0, many0_count, many1, separated_list};
use nom::sequence::{delimited, pair, tuple, preceded, terminated};
use nom::IResult;
use num_bigint::BigUint;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Debug, Display};
use std::iter::FromIterator;
use std::mem::size_of;

use crate::instructions::EvmOpcode;

// must be at least 1 byte but less than or equal to size_of::<usize>
const ADDRESS_SIZE: usize = 2;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Symbol(pub String);

impl Symbol {
    fn new(name: &str) -> Symbol {
        Symbol(name.to_string())
    }
}

#[derive(Clone, Debug, PartialEq)]
struct MacroDefinition {
    name: Symbol,
    params: Option<Vec<Symbol>>,
    body: Program,
}

impl MacroDefinition {
    fn new(name: Symbol, params: Option<Vec<Symbol>>, body: Program) -> MacroDefinition {
        let params = match &params {
            Some(v) => {
                if v.is_empty() {
                    None
                } else {
                    params
                }
            }
            None => None,
        };
        MacroDefinition { name, params, body }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct MacroCall {
    name: Symbol,
    args: Option<Vec<BlockVec>>,
}

impl MacroCall {
    fn new(name: Symbol, args: Option<Vec<BlockVec>>) -> MacroCall {
        let args = match &args {
            Some(v) => {
                if v.is_empty() {
                    None
                } else {
                    args
                }
            }
            None => None,
        };
        MacroCall { name, args }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Directive {
    Call(MacroCall),
    Bytes(Vec<u8>),
    Var(Symbol),
    Label(Symbol),
}

#[derive(Clone, Debug, PartialEq)]
struct Block {
    label: Option<Symbol>,
    directives: Vec<Directive>,
}

impl Block {
    fn default() -> Block {
        Block {
            label: None,
            directives: Vec::new(),
        }
    }

    fn with_label(label: Symbol) -> Block {
        Block {
            label: Some(label),
            directives: Vec::new(),
        }
    }

    fn new(label: Option<Symbol>, directives: Vec<Directive>) -> Block {
        assert!(directives.len() > 0, "at least one directive");
        Block { label, directives }
    }
}

type BlockVec = Vec<Block>;

type MacroArgumentMap = BTreeMap<Symbol, BlockVec>;

type AddressMap = HashMap<Symbol, Vec<u8>>;

#[derive(Clone, Debug, PartialEq)]
struct Program {
    macros: Vec<MacroDefinition>,
    blocks: BlockVec,
}

impl Program {
    fn new(macros: Vec<MacroDefinition>, blocks: BlockVec) -> Program {
        Program { macros, blocks }
    }

    fn with_blocks(blocks: BlockVec) -> Program {
        Program { macros: vec![], blocks }
    }
}

pub struct Error {
    err: Box<ErrorImpl>,
}

impl Error {
    fn syntax(code: ErrorCode) -> Self {
        Error {
            err: Box::new(ErrorImpl { code: code }),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error: {}", self.err.code)
    }
}

struct ErrorImpl {
    code: ErrorCode,
}

enum ErrorCode {
    UndefinedSymbol(Symbol),
    InvalidParse(String),
    InvalidMacroCallArity(Symbol, usize, usize),
    UndefinedParameter(Symbol),
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorCode::UndefinedSymbol(s) => write!(f, "undefined symbol `{}`", s.0),
            ErrorCode::InvalidParse(s) => write!(f, "invalid parse at `{}`", s),
            ErrorCode::InvalidMacroCallArity(s, params, args) => {
                let plural = |x: usize, a: &str, b: &str| if x > 1 { String::from(a) } else { String::from(b) };
                write!(f, "too {} arguments to macro call `{}`, expected {} argument{}, have {} argument{}", 
                    if params < args { "many" } else { "few" },
                    s.0,
                    plural(*params, &params.to_string(), "single"),
                    plural(*params, "s", ""),
                    args,
                    plural(*args, "s", "")
                )
            },
            ErrorCode::UndefinedParameter(s) => {
                write!(f, "undefined macro parameter `${}`", s.0)
            }
        }
    }
}

fn comment(i: &str) -> IResult<&str, &str> {
    preceded(char(';'), terminated(take_until("\n"), newline))(i)
}

fn comments(i: &str) -> IResult<&str, &str> {
    recognize(many0_count(terminated(comment, multispace0)))(i)
}

fn blank(i: &str) -> IResult<&str, ()> {
    value((), preceded(multispace0, comments))(i)
}

fn underscore(i: &str) -> IResult<&str, &str> {
    recognize(char('_'))(i)
}

fn symbol(i: &str) -> IResult<&str, Symbol> {
    map(
        recognize(
            pair(
                alt((underscore, alpha1)),
                many0_count(preceded(opt(underscore), alphanumeric1)),
            )
        ),
        |x| Symbol::new(x)
    )(i)
}

fn parameter(i: &str) -> IResult<&str, Symbol> {
    preceded(char('$'), symbol)(i)
}

fn macro_parameters(i: &str) -> IResult<&str, Vec<Symbol>> {
    preceded(
        blank,
        delimited(
            char('('),
            separated_list(delimited(blank, char(','), blank), parameter),
            char(')')
        )
    )(i)
}

fn macro_arguments(i: &str) -> IResult<&str, Vec<BlockVec>> {
    preceded(
        blank,
        delimited(
            char('('),
            separated_list(delimited(blank, char(','), blank), many1(block)),
            char(')')
        )
    )(i)
}

fn macro_identifier(i: &str) -> IResult<&str, Symbol> {
    terminated(symbol, not(char(':')))(i)
}

fn macro_call(i: &str) -> IResult<&str, MacroCall> {
    map(
        pair(
            macro_identifier,
            opt(macro_arguments)
        ),
        |(id, args)| MacroCall::new(id, args)
    )(i)
}

fn macro_variable(i: &str) -> IResult<&str, Symbol> {
    preceded(blank, preceded(char('$'), symbol))(i)
}

fn decode_hex_str(s: &str) -> Vec<u8> {
    let mut result = vec![];
    let index = s.len() % 2;
    if index == 1 {
        result.push(u8::from_str_radix(&s[0..1], 16).unwrap());
    }
    for i in (index..s.len()).step_by(2) {
        result.push(u8::from_str_radix(&s[i..(i+2)], 16).unwrap());
    }
    result
}

fn raw_bytes(i: &str) -> IResult<&str, Vec<u8>> {
    map(preceded(preceded(blank, tag("0x")), hex_digit1), decode_hex_str)(i)
}

fn encode_radix(s: &str, radix: u32) -> Vec<u8> {
    let big = BigUint::parse_bytes(s.as_bytes(), radix).unwrap();
    let mut bytes = big.to_bytes_le();
    bytes.push(EvmOpcode::PUSH1 as u8 - 1 + bytes.len() as u8);
    bytes.reverse();
    bytes
}

fn hexadecimal_lit(i: &str) -> IResult<&str, Vec<u8>> {
    map(terminated(hex_digit1, char('h')), |x| encode_radix(x, 16))(i)
}

fn octal_lit(i: &str) -> IResult<&str, Vec<u8>> {
    map(terminated(oct_digit1, char('o')), |x| encode_radix(x, 8))(i)
}

fn bin_digit1(i: &str) -> IResult<&str, &str> {
    take_while1(|x| x == '0' || x == '1')(i)
}

fn binary_lit(i: &str) -> IResult<&str, Vec<u8>> {
    map(terminated(bin_digit1, char('b')), |x| encode_radix(x, 2))(i)
}

fn decimal_lit(i: &str) -> IResult<&str, Vec<u8>> {
    map(digit1, |x| encode_radix(x, 10))(i)
}

fn literal(i: &str) -> IResult<&str, Vec<u8>> {
    alt((
        hexadecimal_lit,
        octal_lit,
        binary_lit,
        decimal_lit,
    ))(i)
}

fn directive(i: &str) -> IResult<&str, Directive> {
    preceded(
        blank,
        alt((
            map(raw_bytes, Directive::Bytes),
            map(literal, Directive::Bytes),
            map(macro_call, Directive::Call),
            map(macro_variable, Directive::Var),
        ))
    )(i)
}

fn one_liner(i: &str) -> IResult<&str, Program> {
    let (i, j) = not_line_ending(i)?;
    let (_, prog) = program(j)?;
    Ok((i, prog))
}

fn macro_body(i: &str) -> IResult<&str, Program> {
    alt((
        terminated(program, preceded(blank, tag("%end"))),
        one_liner
    ))(i)
}

fn macro_definition(i: &str) -> IResult<&str, MacroDefinition> {
    map(
        preceded(
            tag("%define"),
            tuple((
                preceded(blank, symbol),
                opt(macro_parameters),
                macro_body,
            ))
        ),
        |(name, params, body)| MacroDefinition::new(name, params, body)
    )(i)
}

fn macro_definitions(i: &str) -> IResult<&str, Vec<MacroDefinition>> {
    many0(preceded(blank, macro_definition))(i)
}

fn block(i: &str) -> IResult<&str, Block> {
    map(
        pair(
            opt(preceded(blank, terminated(symbol, char(':')))),
            many1(directive),
        ),
        |(label, mut directives)| {
            if label.is_some() {
                directives.insert(0, Directive::Bytes(vec![EvmOpcode::JUMPDEST as u8]));
            }
            Block::new(label, directives)
        },
    )(i)
}

fn program(i: &str) -> IResult<&str, Program> {
    map(
        pair(
            macro_definitions,
            terminated(many1(block), blank)
        ),
        |(macros, blocks)| {
            Program::new(macros, blocks)
        },
    )(i)
}

fn parse(i: &str) -> Result<Program, Error> {
    match program(i) {
        Ok((i, program)) => {
            if i.is_empty() {
                Ok(program)
            }
            else {
                Err(Error::syntax(ErrorCode::InvalidParse(i.to_string())))
            }
        },
        Err(_) => {
            Err(Error::syntax(ErrorCode::InvalidParse(i.to_string())))
        }
    }
}

fn build_opcodes() -> String {
    EvmOpcode::iter().map(|x| {
        format!("%define {}() {:#02x}", x, *x as u8)
    })
    .collect::<Vec<_>>()
    .join("\n")
}

fn build_argument_map(
    macro_name: &Symbol,
    params: &Option<Vec<Symbol>>,
    args: &Option<Vec<BlockVec>>,
) -> Result<Option<MacroArgumentMap>, Error> {
    match params {
        Some(params) => match args {
            Some(args) => {
                if params.len() == args.len() {
                    let mut map = BTreeMap::new();
                    for (param, arg) in params.iter().zip(args.iter()) {
                        map.insert(param.clone(), arg.clone());
                    }
                    Ok(Some(map))
                } else {
                    Err(Error::syntax(ErrorCode::InvalidMacroCallArity(macro_name.clone(), params.len(), args.len())))
                }
            }
            None => Err(Error::syntax(ErrorCode::InvalidMacroCallArity(macro_name.clone(), params.len(), 0)))
        },
        None => match args {
            None => Ok(None),
            Some(args) => Err(Error::syntax(ErrorCode::InvalidMacroCallArity(macro_name.clone(), 0, args.len())))
        },
    }
}

fn invoke_macros(
    block: &Block,
    macros: &Vec<MacroDefinition>,
    arg_map: &Option<MacroArgumentMap>,
    mut new_blocks: BlockVec
) -> Result<BlockVec, Error> {
    let macro_map: HashMap<_, _> = macros.iter().map(|m| (&m.name, m)).collect();
    if let Some(l) = &block.label {
        // creates a new block with same label than input block
        new_blocks.push(Block::with_label(l.clone()));
    } else {
        // creates a new block if new_blocks is still empty
        if new_blocks.is_empty() {
            new_blocks.push(Block::default());
        }
    }
    
    // this is now safe to unwrap last elements of new_blocks
    block.directives.iter().fold(Ok(new_blocks), |result: Result<BlockVec, Error>, d| {
        result.and_then(|mut new_blocks| {
            match d {
                Directive::Call(call) => match macro_map.get(&call.name) {
                    Some(def) => {
                        build_argument_map(&call.name, &def.params, &call.args).and_then(|map| {
                            let blocks = &def.body.blocks;
                            blocks.iter().fold(Ok(new_blocks), |result, b| {
                                result.and_then(|new_blocks| invoke_macros(&b, &macros, &map, new_blocks))
                            })
                        })
                    }
                    None => {
                        new_blocks
                            .last_mut()
                            .unwrap()
                            .directives
                            .push(Directive::Label(call.name.clone()));
                        Ok(new_blocks)
                    }
                },
                Directive::Bytes(b) => {
                    new_blocks
                        .last_mut()
                        .unwrap()
                        .directives
                        .push(Directive::Bytes(b.to_vec()));
                    Ok(new_blocks)
                }
                Directive::Var(s) => {
                    match arg_map {
                        Some(map) => {
                            map.get(s).map_or(Err(Error::syntax(ErrorCode::UndefinedParameter(s.clone()))), |blocks| {
                                blocks.iter().fold(Ok(new_blocks), |result, b| {
                                    result.and_then(|new_blocks| invoke_macros(&b, &macros, &None, new_blocks))
                                })
                            })
                        }
                        None => {
                            Err(Error::syntax(ErrorCode::UndefinedParameter(s.clone())))
                        }
                    }
                },
                Directive::Label(_) => unreachable!(),
            }
        })
    })
}

fn expand_macros(program: Program) -> Result<Program, Error> {
    program.blocks.iter().fold(Ok(vec![]), |result, b| {
        result.and_then(|new_blocks| invoke_macros(&b, &program.macros, &None, new_blocks))
    })
    .and_then(|blocks| Ok(Program::with_blocks(blocks)))
}

fn build_block_addresses(blocks: &BlockVec) -> Vec<usize> {
    blocks
        .iter()
        .map(|b| {
            b.directives.iter().fold(0, |acc, d| {
                let size = match d {
                    Directive::Bytes(v) => v.len(),
                    Directive::Label(_) => 1 + ADDRESS_SIZE,
                    _ => panic!("macros have not been expanded yet"),
                };
                acc + size
            })
        })
        .scan(0, |sum, i| {
            let s = *sum;
            *sum += i;
            Some(s)
        })
        .collect()
}

fn build_label_addresses(blocks: &BlockVec) -> AddressMap {
    let addresses = build_block_addresses(&blocks);
    HashMap::from_iter(
        blocks
            .into_iter()
            .enumerate()
            .filter_map(|(i, b)| match &b.label {
                Some(_) => Some((i, b.clone())),
                None => None,
            })
            .map(|(i, b)| {
                const BITMASK: usize = usize::max_value() >> ((size_of::<usize>()-ADDRESS_SIZE) * 8);
                let address = addresses[i];
                if (address & !BITMASK) > 0 {
                    panic!("address too large to fit on {} byte{}",
                        ADDRESS_SIZE,
                        if ADDRESS_SIZE > 1 { "s" } else { "" }
                    );
                }
                let bytes = address.to_be_bytes();
                let code = EvmOpcode::PUSH1 as u8 + (ADDRESS_SIZE-1) as u8;
                let mut instr: Vec<u8> = vec![code];
                instr.extend_from_slice(&bytes[bytes.len() - ADDRESS_SIZE..]);
                (b.label.unwrap(), instr)
            }),
    )
}

fn find_undefined_label(blocks: &BlockVec, map: &AddressMap) -> Option<Symbol> {
    blocks
        .iter()
        .flat_map(|b| b.directives.iter())
        .filter_map(|d| match &d {
            Directive::Label(l) => Some(l.clone()),
            _ => None,
        })
        .find_map(|l| {
            if map.get(&l).is_none() {
                Some(l.clone())
            } else {
                None
            }
        })
}

fn flatten_blocks(program: Program) -> Result<Vec<u8>, Error> {
    let blocks = &program.blocks;
    let addresses = build_label_addresses(blocks);
    if let Some(l) = find_undefined_label(blocks, &addresses) {
        return Err(Error::syntax(ErrorCode::UndefinedSymbol(l.clone())));
    }
    let bytecode: Vec<u8> = blocks
        .into_iter()
        .flat_map(|b| {
            (&b.directives).into_iter().flat_map(|d| match d {
                Directive::Bytes(v) => v.clone().into_iter(),
                Directive::Label(l) => addresses.get(l).unwrap().clone().into_iter(),
                _ => panic!("macros have not been expanded yet"),
            })
        })
        .collect();
    Ok(bytecode)
}

const PRELUDE: &str = "
%define retword()
    0h
    MSTORE
    20h
    0h
    RETURN
%end
";

pub fn from_string(input: &str) -> Result<Vec<u8>, Error> {
    let input = format!("{}\n{}\n{}", build_opcodes(), PRELUDE, input);
    let result = parse(&input)
        .and_then(expand_macros)
        .and_then(flatten_blocks);
    result
}
