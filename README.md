# Brainfuck Interpreter

It's a brainfuck interpreter.

## Usage Instructions
```
**Usage**: ./bfi [OPTIONS] [FILE]

**Arguments**:
  [FILE]  An input file

**Options**:
  -c, --inline <INLINE_CODE>  Use this to specify inline Brainfuck code [default: ""]
      --debug                 This is debug mode for debugging the interpreter
  -h, --help                  Print help
  ```

--inline: Takes in a string as BF code to interpret.

--debug: Prints a debug log containing the program state after every instruction execution.
