pub(crate) mod tests;

use std::{fs::{self}, io::stdin};
use clap::{CommandFactory, Parser};

#[derive(Debug)]
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

fn probe_loop(bf_code: &str, start_idx: Option<usize>, end_idx: Option<usize>) -> Result<usize, ()> {
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
            panic!("Logic Error in probe_loop: Use either start_idx or end_idx, not both! This error should not happen during normal interpreter use.")
        },
    }
}

fn parse_bf(bf_code: &str) -> Vec<BfInstructions> {
    if bf_code.is_empty() { return vec![] }

    let mut instructions: Vec<BfInstructions> = Vec::new();

    let mut row_number = 1;
    let mut col_number = 0;

    for (i, c) in bf_code.chars().enumerate() {
        if c == '\n' {
            row_number += 1;
            col_number = 0;
        }
        col_number += 1;
        if let Some(inst) = BfInstructions::from_char(c) {
            match inst {
                BfInstructions::LoopStartUnprobed => {
                    match probe_loop(bf_code, Some(i), None) {
                        Ok(v) => instructions.push(BfInstructions::LoopStart(v)),
                        Err(_) => panic!("Syntax Error: Unclosed loop-start at row {}, col {}", row_number, col_number)
                    }
                }
                BfInstructions::LoopEndUnprobed => {
                    match probe_loop(bf_code, None, Some(i)) {
                        Ok(v) => instructions.push(BfInstructions::LoopEnd(v)),
                        Err(_) => panic!("Syntax Error: Unexpected loop-end (']') at row {}, col {}", row_number, col_number)
                    }
                }
                _ => {
                    instructions.push(inst)
                }
            }
        }
    }

    instructions


}

fn execute(instructions: Vec<BfInstructions>, debug: bool) -> (Vec<i32>, String) {

    let mut all_output = String::new();

    let mut read_buf = String::new();
    let mut input_queue = String::new();
    let mut tape: Vec<i32> = vec![0; 1];
    let mut ptr = 0;
    let mut instruction_ptr = 0;

    while instruction_ptr < instructions.len() {
        match instructions[instruction_ptr] {
            BfInstructions::PtrIncrement => {
                if debug { println!("Ptr >, tape: {:?}, ptr: {ptr}", tape); }
                ptr += 1;
                if ptr >= tape.len() {
                    tape.push(0);
                }
            }
            BfInstructions::PtrDecrement => {
                if debug { println!("Ptr <"); }
                if ptr == 0 {
                    panic!("Pointer Error: Attempted to decrease pointer past 0!")
                }
                ptr -= 1;
            }
            BfInstructions::ValIncrement => {
                tape[ptr] += 1;
                if debug { println!("val +, tape: {:?}, ptr: {ptr}", tape); }
            }
            BfInstructions::ValDecrement => {
                tape[ptr] -= 1;
                if debug { println!("val -, tape: {:?}, ptr: {ptr}", tape); }
            }
            BfInstructions::Read => {
                if debug { print!("reading>"); }
                if let Some(c) = read_buf.chars().next() {
                    tape[ptr] = c as i32;
                } else if let Ok(_line) = stdin().read_line(&mut read_buf) {
                    input_queue += &read_buf;
                } else {
                    tape[ptr] = -1;
                }
                if debug { println!("tape: {:?}, ptr: {ptr}", tape)}
            }
            BfInstructions::Write => {
                if debug { print!("writing: "); }
                if let Some(c) = char::from_u32((tape[ptr] % 256).try_into().unwrap()) {
                    print!("{}", c);
                    all_output.push(c)
                }
                if debug { println!(); }
            }
            BfInstructions::LoopStart(i) => {
                if debug { println!("hit loop-start at: {instruction_ptr}, end at {i}, tape: {:?}, ptr: {ptr}", tape); }
                if tape[ptr] == 0 {
                    instruction_ptr = i
                }
            }
            BfInstructions::LoopEnd(i) => {
                if debug { println!("hit loop-end at {instruction_ptr}, start at {i}, tape: {:?}, ptr: {ptr}", tape); }
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

    // execute Bf
    execute(instructions, args.debug);
}