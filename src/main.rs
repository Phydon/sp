// use aho_corasick::AhoCorasick; // TODO remove?
use clap::{Arg, ArgAction, Command};
use flexi_logger::{detailed_format, Duplicate, FileSpec, Logger};
use log::error;
use owo_colors::colored::*;
use rayon::prelude::*;
use regex::RegexSet;

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
        if let Some(patterns) = matches
            .get_many::<String>("patterns")
            .map(|a| a.collect::<Vec<_>>())
        {
            let re = RegexSet::new(patterns).unwrap();
            // let literal = pattern;

            let pipe = read_pipe();

            if parallel_flag {
                let lines = par_split_pipe_by_lines(pipe);
                lines.into_par_iter().for_each(|line| {
                    // todo!();
                    println!("{line}");
                })
            } else {
                let lines = split_pipe_by_lines(pipe);
                lines.into_iter().for_each(|line| {
                    // todo!();
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

// TODO
fn search_regex(patterns: Vec<String>, reg: RegexSet) {
    todo!();
}

// TODO
// fn highlight_pattern_in_line(line: &str, config: &Config) -> String {
//     // find first byte of pattern in filename
//     let pat_in_file = line.find(&config.pattern).unwrap_or_else(|| 9999999999);

//     if pat_in_file == 9999999999 {
//         // if no pattern found return just the filename
//         return line.to_string();
//     } else {
//         let first_from_name = &line[..pat_in_file];
//         let last_from_name = &line[(pat_in_file + config.pattern.len())..];
//         // colourize the pattern in the filename
//         let highlighted_pattern = config.pattern.truecolor(112, 110, 255).to_string();

//         let mut result = String::from(first_from_name);
//         result.push_str(&highlighted_pattern);
//         result.push_str(last_from_name);

//         result.to_string()
//     }
// }

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
        .version("1.0.0")
        .author("Leann Phydon <leann.phydon@gmail.com>")
        .arg(
            Arg::new("patterns")
                .help("Enter the search patterns")
                .long_help(format!(
                    "{}\n{}",
                    "Enter the search patterns", "Treat as regex patterns by default",
                ))
                // .trailing_var_arg(true) // TODO needed?
                // .value_terminator(";") // TODO needed?
                .action(ArgAction::Append)
                .value_name("PATTERNS"),
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
todo
    "###
    );

    println!("\n{}\n----------", "Example 2".bold());
    println!(
        r###"
todo
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
