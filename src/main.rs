use clap::{Arg, ArgAction, Command};
use flexi_logger::{detailed_format, Duplicate, FileSpec, Logger};
use log::error;
use owo_colors::colored::*;
use rayon::prelude::*;
use regex::Regex;

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

    // get config dir
    let config_dir = check_create_config_dir().unwrap_or_else(|err| {
        error!("Unable to find or create a config directory: {err}");
        process::exit(1);
    });

    // initialize the logger
    let _logger = Logger::try_with_str("info") // log warn and error
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

    // handle arguments
    let matches = sp().get_matches();
    let parallel_flag = matches.get_flag("parallel");
    let matches_flag = matches.get_flag("matches");

    if let Some(_) = matches.subcommand_matches("log") {
        if let Ok(logs) = show_log_file(&config_dir) {
            println!("{}", "Available logs:".bold().yellow());
            println!("{}", logs);
        } else {
            error!("Unable to read logs");
            process::exit(1);
        }
    } else if let Some(_) = matches.subcommand_matches("examples") {
        examples();
    } else {
        if let Some(pattern) = matches.get_one::<String>("pattern") {
            let re = Regex::new(pattern).unwrap();
            // let literal = pattern;

            let pipe = read_pipe();

            if parallel_flag {
                let lines = par_split_pipe_by_lines(pipe);
                lines.into_par_iter().for_each(|line| {
                    let captures = search_regex(&line, re.clone());
                    if let Some(high_line) = highlight_pattern_in_line(line, captures, matches_flag)
                    {
                        println!("{}", high_line);
                    }
                })
            } else {
                let lines = split_pipe_by_lines(pipe);
                lines.into_iter().for_each(|line| {
                    let captures = search_regex(&line, re.clone());
                    if let Some(high_line) = highlight_pattern_in_line(line, captures, matches_flag)
                    {
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

fn search_regex(hay: &str, reg: Regex) -> Vec<(String, usize, usize)> {
    let mut captures = Vec::new();
    let matches = reg.find_iter(hay);

    for mat in matches {
        captures.push((mat.as_str().to_owned(), mat.start(), mat.end()));
    }

    captures
}

fn highlight_pattern_in_line(
    line: String,
    captures: Vec<(String, usize, usize)>,
    matches_flag: bool,
) -> Option<String> {
    if captures.is_empty() {
        if matches_flag {
            return None;
        } else {
            return Some(line);
        }
    }

    let mut high_line = String::new();
    // keep track of current position in current line to handle multiple captures
    let mut current_position = 0;
    let mut counter = 1; // start at 1 because cap_len >= 1
    let cap_len = captures.len();
    for cap in captures {
        let pattern = cap.0;
        let pat_start = cap.1;
        let pat_end = cap.2;

        let till_pat = &line[current_position..pat_start];
        let high_pat = pattern.truecolor(112, 110, 255).to_string();

        high_line.push_str(till_pat);
        high_line.push_str(&high_pat);

        // no more captures -> add rest of the line to high_line
        if counter.eq(&cap_len) {
            let pat_till_end = &line[pat_end..];
            high_line.push_str(pat_till_end);
            break;
        }

        current_position = pat_end;
        counter += 1;
    }

    return Some(high_line);
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
            "XA".bold().truecolor(250, 0, 104),
            "Leann Phydon <leann.phydon@gmail.com>".italic().dimmed()
        ))
        .long_about(format!("{}", "Search in stdin",))
        // TODO update version
        .version("1.0.1")
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

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

    #[test]
    fn search_regex_single_match_test() {
        let pipe = "This is a test";
        let re = Regex::from_str("test").unwrap();
        let captures = search_regex(pipe, re);
        let expected = vec![("test".to_string(), 10, 14)];
        assert_eq!(captures, expected);
    }

    #[test]
    fn search_regex_multi_match_test() {
        let pipe = "This is a test, with a lot of testing results";
        let re = Regex::from_str("test").unwrap();
        let captures = search_regex(pipe, re);
        let expected = vec![("test".to_string(), 10, 14), ("test".to_string(), 30, 34)];
        assert!(captures.len() == 2);
        assert_eq!(captures, expected);
    }
}
