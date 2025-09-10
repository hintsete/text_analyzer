use std::collections::HashMap;
use std::env;
use std::fs;
use std::process;


#[derive(Debug)]
enum CliError {
    MissingFilePath,
    InvalidMinLength { value: String, reason: String },
    InvalidStartsWith { value: String, reason: String },
    FileNotFound(String),
    FileReadPermission(String),
    EmptyFile,
}

#[derive(Default)]
struct Config {
    file_path: String,
    min_length: usize,
    starts_with: Option<char>,
}

impl Config {
    fn new(args: Vec<String>) -> Result<Self, CliError> {
        let mut config = Config::default();
        
        if args.len() < 2 {
            return Err(CliError::MissingFilePath);
        }
        config.file_path = args[1].clone();

        let mut i = 2;
        while i < args.len() {
            match args[i].as_str() {
                "--min-length" => {
                    i += 1;
                    config.min_length = args
                        .get(i)
                        .ok_or_else(|| CliError::InvalidMinLength {
                            value: "".to_string(),
                            reason: "Missing value".to_string(),
                        })?
                        .parse()
                        .map_err(|_| CliError::InvalidMinLength {
                            value: args[i].clone(),
                            reason: "Not a number".to_string(),
                        })?;
                }
                "--starts-with" => {
                    i += 1;
                    let c = args
                        .get(i)
                        .ok_or_else(|| CliError::InvalidStartsWith {
                            value: "".to_string(),
                            reason: "Missing value".to_string(),
                        })?
                        .chars()
                        .next()
                        .ok_or_else(|| CliError::InvalidStartsWith {
                            value: args[i].clone(),
                            reason: "Not a char".to_string(),
                        })?;
                    if !c.is_alphabetic() {
                        return Err(CliError::InvalidStartsWith {
                            value: args[i].clone(),
                            reason: "Must be a letter".to_string(),
                        });
                    }
                    config.starts_with = Some(c.to_ascii_lowercase());
                }
                _ => {}
            }
            i += 1;
        }
        Ok(config)
    }
}

fn main() {
    if let Err(err) = run() {
        process::exit(err.into());
    }
}
fn run() -> Result<(), CliError> {
    
    let args: Vec<String> = env::args().collect();
    let config = Config::new(args)?;

    let text = fs::read_to_string(&config.file_path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => CliError::FileNotFound(config.file_path.clone()),
        std::io::ErrorKind::PermissionDenied => CliError::FileReadPermission(config.file_path.clone()),
        _ => CliError::FileReadError(e.to_string()),
    })?;
    if text.trim().is_empty() {
        return Err(CliError::EmptyFile);
    }

    let min_filter = |min_len: usize| move |word: &str| word.len() > min_len;
    let starts_filter = |c: Option<char>| move |word: &str| {
        c.map_or(true, |c| {
            word.chars()
                .next()
                .map_or(false, |first| first.to_ascii_lowercase() == c)
        })
    };
    let combined_filter = |word: &str| min_filter(config.min_length)(word) && starts_filter(config.starts_with)(word);

    let freq: HashMap<String, u32> = text
        .split_whitespace()
        .map(|w| w.to_lowercase())
        .filter(|w| !w.is_empty())
        .filter(combined_filter)
        .fold(HashMap::new(), |mut acc, word| {
            *acc.entry(word).or_insert(0) += 1;
            acc
        });

    let total_words: u32 = freq.values().sum();
    let unique_words = freq.len();
    let most_common = freq
        .iter()
        .max_by_key(|&(word, &count)| (count, std::cmp::Reverse(word)));

    println!("Total word count: {}", total_words);
    println!("Number of unique words: {}", unique_words);
    match most_common {
        Some((word, &count)) => println!("Most common word: \"{}\" with count {}", word, count),
        None => println!("No words found."),
    }

    Ok(())
}

impl From<CliError> for i32 {
    fn from(err: CliError) -> i32 {
        match err {
            CliError::MissingFilePath => { eprintln!("Error: Missing file path."); 1 }
            CliError::InvalidMinLength { value, reason } => {
                eprintln!("Error: Invalid --min-length '{}': {}", value, reason);
                2
            }
            CliError::InvalidStartsWith { value, reason } => {
                eprintln!("Error: Invalid --starts-with '{}': {}", value, reason);
                3
            }
            CliError::FileNotFound(path) => { eprintln!("Error: File '{}' not found.", path); 4 }
            CliError::FileReadPermission(path) => {
                eprintln!("Error: Permission denied reading '{}'.", path);
                5
            }
            CliError::EmptyFile => { eprintln!("Error: File is empty."); 6 }
        }
    }
}