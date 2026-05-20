# Brainfuck Interpreter

A Brainfuck interpreter written in Rust using Cargo.

## Features

- Execute Brainfuck programs from a file
- Execute inline Brainfuck code from the command line
- Optional debug mode with instruction-by-instruction state logging

## Requirements

- Rust
- Cargo

Install Rust and Cargo from https://rustup.rs if they are not already installed.

## Building

Clone the repository and build the project with Cargo:

```bash
cargo build --release
```

The compiled binary will be located at:

```bash
target/release/bfi
```

## Usage

```text
Usage: ./bfi [OPTIONS] [FILE]

Arguments:
  [FILE]  An input file containing Brainfuck code

Options:
  -c, --inline <INLINE_CODE>  Execute inline Brainfuck code
      --debug                 Enable debug logging
  -h, --help                  Print help information
```

## Examples

### Run a Brainfuck file

```bash
./bfi hello.bf
```

### Run inline Brainfuck code

```bash
./bfi --inline "+++++[>++++++++<-]>+."
```

### Run with debug mode enabled

```bash
./bfi hello.bf --debug
```

## Options

### `--inline`

Executes the provided string as Brainfuck code directly from the command line.

Example:

```bash
./bfi --inline "++++[>++++<-]>+."
```

### `--debug`

Prints a debug log showing the interpreter state after every instruction execution.

This may include:

- Instruction pointer position
- Memory pointer position
- Current memory tape state
- Current instruction being executed

Example:

```bash
./bfi program.bf --debug
```
