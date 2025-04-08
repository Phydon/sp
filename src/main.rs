use clap::{Arg, ArgAction, Command};
use flexi_logger::{detailed_format, Duplicate, FileSpec, Logger};
use log::error;
use owo_colors::colored::*;
use rayon::prelude::*;
use regex::{Match, Regex};

use std::{
    fs,
    io::{self, BufRead},
    path::{Path, PathBuf},
    process,
};

fn main() {
    // handle Ctrl+C
    ctrlc::set_handler(move || {
        println!("{}", "Received Ctrl-C!".italic(),);
        process::exit(0)
    })
    .expect("Error setting Ctrl-C handler");

    // initialize the logger
    let config_dir = check_create_config_dir().unwrap_or_else(|err| {
        error!("Unable to find or create a config directory: {err}");
        process::exit(1);
    });

    init_logger(&config_dir);

    // handle arguments
    let matches = sp().get_matches();
    let parallel_flag = matches.get_flag("parallel");
    let matches_flag = matches.get_flag("matches");

    if let Some(_) = matches.subcommand_matches("log") {
        show_logs(&config_dir);
    } else if let Some(_) = matches.subcommand_matches("examples") {
        examples();
    } else if let Some(_) = matches.subcommand_matches("syntax") {
        show_regex_syntax();
    } else {
        if let Some(pattern) = matches.get_one::<String>("pattern") {
            let re = Regex::new(pattern).unwrap();

            let pipe = read_pipe();

            if parallel_flag {
                let lines = par_split_pipe_by_lines(pipe);
                lines.into_par_iter().for_each(|line| {
                    let captures = search_regex(&line, re.clone());
                    if let Some(high_line) = highlight_capture(&line, &captures, matches_flag) {
                        println!("{}", high_line);
                    }
                })
            } else {
                let lines = split_pipe_by_lines(pipe);
                lines.into_iter().for_each(|line| {
                    let captures = search_regex(&line, re.clone());
                    if let Some(high_line) = highlight_capture(&line, &captures, matches_flag) {
                        println!("{}", high_line);
                    }
                })
            }
        } else {
            let _ = sp().print_help();
            process::exit(0);
        }
    }
}

fn read_pipe() -> String {
    let mut input = io::stdin()
        .lock()
        .lines()
        .fold("".to_string(), |acc, line| acc + &line.unwrap() + "\n");

    let _ = input.pop();

    input.trim().to_string()
}

fn split_pipe_by_lines(pipe: String) -> Vec<String> {
    // handle multiple lines in stdin
    pipe.lines()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
}

fn par_split_pipe_by_lines(pipe: String) -> Vec<String> {
    // handle multiple lines in stdin in parallel
    pipe.par_lines()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
}

fn search_regex(hay: &str, reg: Regex) -> Vec<Match> {
    let captures: Vec<_> = reg.find_iter(hay).collect();

    captures
}

fn highlight_capture(line: &str, captures: &Vec<Match>, matches_flag: bool) -> Option<String> {
    if captures.is_empty() {
        if matches_flag {
            return None;
        } else {
            return Some(line.to_string());
        }
    }

    // pre-allocate enough memory for original line + estimated additional space for ANSI codes (est. each color adds ~20 bytes)
    // this reduces the number of times the string's buffer needs to be reallocated as elements are added
    let mut new = String::with_capacity(line.len() + captures.len() * 20);

    let mut last_match = 0;
    for cap in captures {
        new.push_str(&line[last_match..cap.start()]);

        let pattern = cap.as_str().truecolor(112, 110, 255).to_string();
        new.push_str(&pattern);

        last_match = cap.end();
    }
    new.push_str(&line[last_match..]);

    Some(new)
}

// build cli
fn sp() -> Command {
    Command::new("sp")
        .bin_name("sp")
        .before_help(format!(
            "{}\n{}",
            "SP".bold().truecolor(250, 0, 104),
            "Leann Phydon <leann.phydon@gmail.com>".italic().dimmed()
        ))
        .about("Search in stdin")
        .before_long_help(format!(
            "{}\n{}",
            "SP".bold().truecolor(250, 0, 104),
            "Leann Phydon <leann.phydon@gmail.com>".italic().dimmed()
        ))
        .long_about(format!("{}", "Search in stdin",))
        // TODO update version
        .version("1.0.4")
        .author("Leann Phydon <leann.phydon@gmail.com>")
        .arg(
            Arg::new("matches")
                .short('m')
                .long("matches")
                .help("Show only lines that contain at least one match")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("pattern")
                .help("Enter the search pattern")
                .long_help(format!(
                    "{}\n{}",
                    "Enter the search pattern", "Treat as regex pattern by default",
                ))
                .action(ArgAction::Set)
                .value_name("PATTERN"),
        )
        .arg(
            Arg::new("parallel")
                .short('p')
                .long("parallel")
                .help("Process input in parallel if possible")
                .long_help(format!(
                    "{}\n{}",
                    "Process input in parallel if possible",
                    "The input order will most likely change"
                ))
                .action(ArgAction::SetTrue),
        )
        .subcommand(
            Command::new("examples")
                .long_flag("examples")
                .about("Show examples"),
        )
        .subcommand(
            Command::new("log")
                .short_flag('L')
                .long_flag("log")
                .about("Show content of the log file"),
        )
        .subcommand(
            Command::new("syntax")
                .short_flag('S')
                .long_flag("syntax")
                .about("Show regex syntax information"),
        )
}

fn examples() {
    println!("\n{}\n----------", "Example 1".bold());
    println!(
        r###"
- this highlights the word 'test' 

$ echo "this is a test" | sp test

this is a test
    "###
    );

    println!("\n{}\n----------", "Example 2".bold());
    println!(
        r###"
- show only matching lines

$ echo "first test" "second nothing" "third test" | sp test -m

first test
third test
    "###
    );
}

fn show_regex_syntax() {
    println!("{}", "Regex Syntax".bold().blue());
    println!(
        "More information on '{}'",
        "https://docs.rs/regex/latest/regex/#syntax".italic()
    );
    println!("\n{}", "Matching one character:".bold());
    println!(
        r###"
.             any character except new line (includes new line with s flag)
[0-9]         any ASCII digit
\d            digit (\p{{Nd}})
\D            not digit
\pX           Unicode character class identified by a one-letter name
\p{{Greek}}     Unicode character class (general category or script)
\PX           Negated Unicode character class identified by a one-letter name
\P{{Greek}}     negated Unicode character class (general category or script)
        "###
    );
    println!("\n{}", "Character classes:".bold());
    println!(
        r###"
[xyz]         A character class matching either x, y or z (union).
[^xyz]        A character class matching any character except x, y and z.
[a-z]         A character class matching any character in range a-z.
[[:alpha:]]   ASCII character class ([A-Za-z])
[[:^alpha:]]  Negated ASCII character class ([^A-Za-z])
[x[^xyz]]     Nested/grouping character class (matching any character except y and z)
[a-y&&xyz]    Intersection (matching x or y)
[0-9&&[^4]]   Subtraction using intersection and negation (matching 0-9 except 4)
[0-9--4]      Direct subtraction (matching 0-9 except 4)
[a-g~~b-h]    Symmetric difference (matching `a` and `h` only)
[\[\]]        Escaping in character classes (matching [ or ])
[a&&b]        An empty character class matching nothing        
        "###
    );
    println!("\n{}", "Composites:".bold());
    println!(
        r###"
xy    concatenation (x followed by y)
x|y   alternation (x or y, prefer x)
        "###
    );
    println!("\n{}", "Repetitions:".bold());
    println!(
        r###"
x*        zero or more of x (greedy)
x+        one or more of x (greedy)
x?        zero or one of x (greedy)
x*?       zero or more of x (ungreedy/lazy)
x+?       one or more of x (ungreedy/lazy)
x??       zero or one of x (ungreedy/lazy)
x{{n,m}}    at least n x and at most m x (greedy)
x{{n,}}     at least n x (greedy)
x{{n}}      exactly n x
x{{n,m}}?   at least n x and at most m x (ungreedy/lazy)
x{{n,}}?    at least n x (ungreedy/lazy)
x{{n}}?     exactly n x        
        "###
    );
    println!("\n{}", "Empty matches:".bold());
    println!(
        r###"
^               the beginning of a haystack (or start-of-line with multi-line mode)
$               the end of a haystack (or end-of-line with multi-line mode)
\A              only the beginning of a haystack (even with multi-line mode enabled)
\z              only the end of a haystack (even with multi-line mode enabled)
\b              a Unicode word boundary (\w on one side and \W, \A, or \z on other)
\B              not a Unicode word boundary
\b{{start}}, \<   a Unicode start-of-word boundary (\W|\A on the left, \w on the right)
\b{{end}}, \>     a Unicode end-of-word boundary (\w on the left, \W|\z on the right))
\b{{start-half}}  half of a Unicode start-of-word boundary (\W|\A on the left)
\b{{end-half}}    half of a Unicode end-of-word boundary (\W|\z on the right)        
        "###
    );
    println!("\n{}", "Grouping:".bold());
    println!(
        r###"
(exp)          numbered capture group (indexed by opening parenthesis)
(?P<name>exp)  named (also numbered) capture group (names must be alpha-numeric)
(?<name>exp)   named (also numbered) capture group (names must be alpha-numeric)
(?:exp)        non-capturing group
(?flags)       set flags within current group
(?flags:exp)   set flags for exp (non-capturing)        
        "###
    );
    println!("\n{}", "Flags:".bold());
    println!(
        r###"
i     case-insensitive: letters match both upper and lower case
m     multi-line mode: ^ and $ match begin/end of line
s     allow . to match \n
R     enables CRLF mode: when multi-line mode is enabled, \r\n is used
U     swap the meaning of x* and x*?
u     Unicode support (enabled by default)
x     verbose mode, ignores whitespace and allow line comments (starting with `#`)        
        "###
    );
    println!("\n{}", "Escape sequences:".bold());
    println!(
        r###"
\*              literal *, applies to all ASCII except [0-9A-Za-z<>]
\a              bell (\x07)
\f              form feed (\x0C)
\t              horizontal tab
\n              new line
\r              carriage return
\v              vertical tab (\x0B)
\A              matches at the beginning of a haystack
\z              matches at the end of a haystack
\b              word boundary assertion
\B              negated word boundary assertion
\b{{start}}, \<   start-of-word boundary assertion
\b{{end}}, \>     end-of-word boundary assertion
\b{{start-half}}  half of a start-of-word boundary assertion
\b{{end-half}}    half of a end-of-word boundary assertion
\123            octal character code, up to three digits (when enabled)
\x7F            hex character code (exactly two digits)
\x{{10FFFF}}      any hex character code corresponding to a Unicode code point
\u007F          hex character code (exactly four digits)
\u{{7F}}          any hex character code corresponding to a Unicode code point
\U0000007F      hex character code (exactly eight digits)
\U{{7F}}          any hex character code corresponding to a Unicode code point
\p{{Letter}}      Unicode character class
\P{{Letter}}      negated Unicode character class
\d, \s, \w      Perl character class
\D, \S, \W      negated Perl character class        
        "###
    );
    println!("\n{}", "Perl character classes (unicode friendly):".bold());
    println!(
        r###"
\d     digit (\p{{Nd}})
\D     not digit
\s     whitespace (\p{{White_Space}})
\S     not whitespace
\w     word character (\p{{Alphabetic}} + \p{{M}} + \d + \p{{Pc}} + \p{{Join_Control}})
\W     not word character        
        "###
    );
    println!("\n{}", "ASCII character classes:".bold());
    println!(
        r###"
[[:alnum:]]    alphanumeric ([0-9A-Za-z])
[[:alpha:]]    alphabetic ([A-Za-z])
[[:ascii:]]    ASCII ([\x00-\x7F])
[[:blank:]]    blank ([\t ])
[[:cntrl:]]    control ([\x00-\x1F\x7F])
[[:digit:]]    digits ([0-9])
[[:graph:]]    graphical ([!-~])
[[:lower:]]    lower case ([a-z])
[[:print:]]    printable ([ -~])
[[:punct:]]    punctuation ([!-/:-@\[-`{{}}-~])
[[:space:]]    whitespace ([\t\n\v\f\r ])
[[:upper:]]    upper case ([A-Z])
[[:word:]]     word characters ([0-9A-Za-z_])
[[:xdigit:]]   hex digit ([0-9A-Fa-f])        
        "###
    );
}

fn check_create_config_dir() -> io::Result<PathBuf> {
    let mut new_dir = PathBuf::new();
    match dirs::config_dir() {
        Some(config_dir) => {
            new_dir.push(config_dir);
            new_dir.push("sp");
            if !new_dir.as_path().exists() {
                fs::create_dir(&new_dir)?;
            }
        }
        None => {
            error!("Unable to find config directory");
        }
    }

    Ok(new_dir)
}

fn init_logger(config_dir: &PathBuf) {
    let _logger = Logger::try_with_str("info") // log info, warn and error
        .unwrap()
        .format_for_files(detailed_format) // use timestamp for every log
        .log_to_file(
            FileSpec::default()
                .directory(&config_dir)
                .suppress_timestamp(),
        ) // change directory for logs, no timestamps in the filename
        .append() // use only one logfile
        .duplicate_to_stderr(Duplicate::Info) // print infos, warnings and errors also to the console
        .start()
        .unwrap();
}

fn show_log_file(config_dir: &PathBuf) -> io::Result<String> {
    let log_path = Path::new(&config_dir).join("sp.log");
    return match log_path.try_exists()? {
        true => Ok(format!(
            "{} {}\n{}",
            "Log location:".italic().dimmed(),
            &log_path.display(),
            fs::read_to_string(&log_path)?
        )),
        false => Ok(format!(
            "{} {}",
            "No log file found:"
                .truecolor(250, 0, 104)
                .bold()
                .to_string(),
            log_path.display()
        )),
    };
}

fn show_logs(config_dir: &PathBuf) {
    if let Ok(logs) = show_log_file(&config_dir) {
        println!("{}", "Available logs:".bold().yellow());
        println!("{}", logs);
    } else {
        error!("Unable to read logs");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_pipe_by_lines_test() {
        let pipe = "This\nis\na\ntest".to_string();
        let result = split_pipe_by_lines(pipe);
        let expected = vec!["This", "is", "a", "test"];
        assert_eq!(result, expected);
    }

    #[test]
    fn par_split_pipe_by_lines_test() {
        let pipe = "This\nis\na\ntest".to_string();
        let result = par_split_pipe_by_lines(pipe);
        let expected = vec!["This", "is", "a", "test"];
        assert!(result.par_iter().any(|x| expected.contains(&x.as_str())));
    }
}
