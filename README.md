[![Tests](https://github.com/Phydon/sp/actions/workflows/rust.yml/badge.svg)](https://github.com/Phydon/sp/actions/workflows/rust.yml)


# sp


***S**earch **P**ipe*

- search for a regex pattern in stdin

## Examples

### Example 1

- this highlights the word 'test'

```$ echo "this is a test" | sp test```

this is a test


### Example 2

- show only matching lines

```$ echo "first test" "second nothing" "third test" | sp test -m```

first test

third test


## Usage

### Short Usage

```
Usage: sp [OPTIONS] [PATTERN] [COMMAND]

Commands:
  examples, --examples  Show examples
  log, -L, --log        Show content of the log file
  syntax, -S, --syntax  Show regex syntax information
  help                  Print this message or the help of the given subcommand(s)

Arguments:
  [PATTERN]  Enter the search pattern

Options:
  -m, --matches   Show only lines that contain at least one match
  -p, --parallel  Process input in parallel if possible
  -h, --help      Print help (see more with '--help')
  -V, --version   Print version
```

### Long Usage

```
Usage: sp [OPTIONS] [PATTERN] [COMMAND]

Commands:
  examples, --examples  Show examples
  log, -L, --log        Show content of the log file
  syntax, -S, --syntax  Show regex syntax information
  help                  Print this message or the help of the given subcommand(s)

Arguments:
  [PATTERN]
          Enter the search pattern
          Treat as regex pattern by default

Options:
  -m, --matches
          Show only lines that contain at least one match

  -p, --parallel
          Process input in parallel if possible
          The input order will most likely change

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Installation

### Windows

via Cargo or get the ![binary](https://github.com/Phydon/sp/releases)

