pub(crate) mod tests;

use std::{collections::VecDeque, fs::{self}, io::{Read, stdin}};
use clap::{CommandFactory, Parser};

#[derive(Clone, Copy, Debug, PartialEq)]
enum BfInstructions {
    PtrIncrement,
    PtrDecrement,
    ValIncrement,
    ValDecrement,
    LoopStartUnprobed,
    LoopEndUnprobed,
    LoopStart(usize), // Value points to corresponding loop end
    LoopEnd(usize), // Value points to corresponding loop start
    Read,
    Write,
    // DebugPrint,
}
impl BfInstructions {
    fn from_char(c: char) -> Option<Self> {
        match c {
            '>' => Some(Self::PtrIncrement),
            '<' => Some(Self::PtrDecrement),
            '+' => Some(Self::ValIncrement),
            '-' => Some(Self::ValDecrement),
            '[' => Some(Self::LoopStartUnprobed),
            ']' => Some(Self::LoopEndUnprobed),
            ',' => Some(Self::Read),
            '.' => Some(Self::Write),
            _ => None,
        }
    }
}

#[derive(Parser, Debug, Clone)]
struct Options {
    /// An input file.
    file: Option<String>,

    /// Use this to specify inline Brainfuck code.
    #[arg(short = 'c', long = "inline", default_value_t = String::new())]
    inline_code: String,

    /// This is debug mode for debugging the interpreter.
    #[arg(long = "debug")]
    debug: bool
}

fn parse_args(args: &Options) -> String {
    // If there's a file name specified, and the file ends in '.b': read from string
    let mut bf_code: String = if let Some(filename) = &args.file {
        if !filename.is_empty() && filename.ends_with(".b") {
            fs::read_to_string(filename).expect("Interpreter Error: Could not read from BF Source code!")
        } else {
            panic!("Interpreter Error: file \"{}\" either does not exist or is not a BF source file (*.b)", filename)
        }
    } else { String::new() };

    if !args.inline_code.is_empty() && bf_code.is_empty() {
        bf_code = args.inline_code.clone()
    }

    // Panic if bf code is still empty
    if bf_code.is_empty() {
        eprintln!("Interpreter Error: No Branfuck source code provided!");
        let _ = Options::command().print_help();
    }
    bf_code
}

fn unclosed_loop_check(bf_code: &str, start_idx: Option<usize>, end_idx: Option<usize>) -> Result<usize, ()> {
    let mut stack_size = 0;
    match (start_idx, end_idx) {
        (Some(start), None) => {
            let mut idx = Err(());
            for (i, c) in bf_code.chars().enumerate().skip(start) {
                match c {
                    '[' => {
                        stack_size += 1
                    },
                    ']' => {
                        stack_size -= 1
                    },
                    _ => {}
                };
                if stack_size < 1 {
                    idx = Ok(i);
                    break;
                }
            };
            idx
        },
        (None, Some(end)) => {
            let mut idx = Err(());

            for (i, c) in bf_code[..=end].char_indices().rev() {
                match c {
                    '[' => {
                        stack_size -= 1
                    },
                    ']' => {
                        stack_size += 1
                    },
                    _ => {}
                }
                if stack_size < 1 {
                    idx = Ok(i);
                    break;
                }
            };
            idx
        },
        _ => {
            panic!("Logic Error in unclosed_loop_check: Use either start_idx or end_idx, not both! This error should not happen during normal interpreter use.")
        },
    }
}

fn parse_bf(bf_code: &str) -> Vec<BfInstructions> {
    if bf_code.is_empty() { return vec![] }

    let mut instructions_unprobed: Vec<BfInstructions> = Vec::new();

    let mut instructions: Vec<BfInstructions> = Vec::new();

    let mut row_number = 1;
    let mut col_number = 0;
    // first pass: Create Bf code with unprobed loops, probe for looping errors
    for (i, c) in bf_code.chars().enumerate() {
        if c == '\n' {
            row_number += 1;
            col_number = 0;
        }
        col_number += 1;
        if let Some(inst) = BfInstructions::from_char(c) {
            instructions_unprobed.push(inst);
            match inst {
                BfInstructions::LoopStartUnprobed => {
                    if unclosed_loop_check(bf_code, Some(i), None).is_err() {
                        panic!("Syntax Error: Unclosed loop-start at row {}, col {}", row_number, col_number)
                    }
                }
                BfInstructions::LoopEndUnprobed => {
                    if unclosed_loop_check(bf_code, None, Some(i)).is_err() {
                        panic!("Syntax Error: Unexpected loop-end (']') at row {}, col {}", row_number, col_number)
                    }
                }
                _ => {}
            }
        }  
    }
    let mut probe_stack: VecDeque<(usize, BfInstructions)> = VecDeque::new();
    for (idx, inst) in instructions_unprobed.iter().enumerate() {
        if *inst == BfInstructions::LoopStartUnprobed {
            probe_stack.push_back((idx, *inst));
            instructions.push(BfInstructions::LoopStartUnprobed);
        } else if *inst == BfInstructions::LoopEndUnprobed
            && let Some((start_idx, _)) = probe_stack.pop_back() {
            instructions[start_idx] = BfInstructions::LoopStart(idx);
            instructions.push(BfInstructions::LoopEnd(start_idx))
        } else {
            instructions.push(*inst)
        }
    }
    instructions


}

fn execute(instructions: Vec<BfInstructions>, debug: bool) -> (Vec<u8>, String) {

    let mut all_output = String::new();

    let mut tape: Vec<u8> = vec![0; 1];
    let mut ptr = 0;
    let mut instruction_ptr = 0;
    let mut stdin = stdin();

    while instruction_ptr < instructions.len() {
        match instructions[instruction_ptr] {
            BfInstructions::PtrIncrement => {
                if debug { println!("Instruction {instruction_ptr}: Ptr >, tape: {:?}, ptr: {ptr}", tape); }
                ptr += 1;
                if ptr >= tape.len() {
                    tape.push(0);
                }
            }
            BfInstructions::PtrDecrement => {
                if debug { println!("Instruction {instruction_ptr}: Ptr <, tape: {:?}, ptr: {ptr}", tape); }
                if ptr == 0 {
                    panic!("Pointer Error: Attempted to decrease pointer past 0!")
                }
                ptr -= 1;
            }
            BfInstructions::ValIncrement => {
                let _ = tape[ptr].wrapping_add(1);
                if debug { println!("Instruction {instruction_ptr}: val +, tape: {:?}, ptr: {ptr}", tape); }
            }
            BfInstructions::ValDecrement => {
                let _ = tape[ptr].wrapping_sub(1);
                if debug { println!("Instruction {instruction_ptr}: val -, tape: {:?}, ptr: {ptr}", tape); }
            }
            BfInstructions::Read => {
                if debug { println!("Instruction {instruction_ptr}: read from stdin, tape: {:?}, ptr: {ptr}", tape); }

                let mut buf = [0u8; 1];

                match stdin.read(&mut buf) {
                    Ok(1) => tape[ptr] = buf[0],
                    Ok(0) => tape[ptr] = 0, // EOF
                    Err(_) => tape[ptr] = 0,
                    _ => unreachable!(),
                }
            }
            BfInstructions::Write => {
                if debug { print!("Instruction {instruction_ptr}: writing: "); }
                    print!("{}", tape[ptr]);
                    all_output.push(tape[ptr] as char);
                
                if debug { println!(); }
            }
            BfInstructions::LoopStart(i) => {
                if debug { println!("Instruction {instruction_ptr}: loop-start, end at {i}, tape: {:?}, ptr: {ptr}", tape); }
                if tape[ptr] == 0 {

                    instruction_ptr = i
                }
            }
            BfInstructions::LoopEnd(i) => {
                if debug { println!("Instruction {instruction_ptr}: loop-end, start at {i}, tape: {:?}, ptr: {ptr}", tape); }
                if tape[ptr] > 0 {
                    instruction_ptr = i
                }
            }
            _ => {
                panic!("Logic error in execute(): Unprobed loop instruction found! This should not happen during normal execution.")
            }
        }
        instruction_ptr += 1
    }
    (tape, all_output)
}

fn main() {
    let args = Options::parse();
    // Read from file / inline code
    let bf_code = parse_args(&args);

    // parse Bf
    let instructions = parse_bf(&bf_code);
    if args.debug {
        println!("Parsed list of bf instructions: {:?}", instructions);
    }

    // execute Bf
    let (tape, output) = execute(instructions, args.debug);

    println!("--- Program finished executing ---");
    println!("Final tape contents: {:?}", tape);
    if args.debug {
        println!("Full program output: {output}");
    }
}