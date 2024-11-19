use anyhow::{anyhow, Result};
use clap::Parser;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::io::{self, Write};
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(
    author = "Isabelle Beaudale <izzyabelle@gmail.com>",
    version = "0.1",
    about = "Interactive group calculator"
)]
struct Args {
    /// Enable test mode for debugging
    #[arg(short, long)]
    test: bool,
}

#[derive(Debug)]
struct Group<T> {
    elements: HashSet<T>,
    operation: fn(&T, &T, usize) -> T,
    identity: T,
}

impl<T> Group<T>
where
    T: Eq + Hash + Clone + Debug,
{
    fn new(elements: HashSet<T>, operation: fn(&T, &T, usize) -> T, identity: T) -> Result<Self> {
        let group = Group {
            elements,
            operation,
            identity,
        };

        group.is_valid_group()?;
        Ok(group)
    }

    fn is_closed(&self) -> bool {
        let modulus = self.elements.len();
        self.elements.iter().all(|x| {
            self.elements
                .iter()
                .all(|y| self.elements.contains(&(self.operation)(x, y, modulus)))
        })
    }

    fn has_identity(&self) -> bool {
        let modulus = self.elements.len();
        self.elements.iter().all(|x| {
            (self.operation)(x, &self.identity, modulus) == *x
                && (self.operation)(&self.identity, x, modulus) == *x
        })
    }

    fn has_inverses(&self) -> bool {
        let modulus = self.elements.len();
        self.elements.iter().all(|a| {
            self.elements.iter().any(|b| {
                (self.operation)(a, b, modulus) == self.identity
                    && (self.operation)(b, a, modulus) == self.identity
            })
        })
    }

    fn is_associative(&self) -> bool {
        let modulus = self.elements.len();
        self.elements.iter().all(|a| {
            self.elements.iter().all(|b| {
                self.elements.iter().all(|c| {
                    (self.operation)(&(self.operation)(a, b, modulus), c, modulus)
                        == (self.operation)(a, &(self.operation)(b, c, modulus), modulus)
                })
            })
        })
    }

    fn is_valid_group(&self) -> Result<()> {
        let mut errors = Vec::new();

        if !self.is_closed() {
            errors.push("  Group is not closed under the operation.");
        }
        if !self.is_associative() {
            errors.push("  Operation is not associative.");
        }
        if !self.has_identity() {
            errors.push("  No valid identity element found.");
        }
        if !self.has_inverses() {
            errors.push("  Not all elements have inverses.");
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow!(errors.join("\n")))
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.test {
        println!("Test mode enabled.");
    }

    match run() {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}\n", e);
            Err(e)
        }
    }
}

const MAIN_PROMPT: &str = "Command (type help to list commands):\n  ";
const CMD_ADD: &str = "add";
const CMD_IDENTITY: &str = "identity";
const CMD_LIST: &str = "list";
const CMD_HELP: &str = "help";
const CMD_CREATE: &str = "create";
const CMD_EXIT: &str = "exit";

fn run() -> Result<()> {
    let mut elements = HashSet::new();
    let mut identity = None;
    let operation = |a: &i32, b: &i32, m: usize| (a + b) % m as i32;

    loop {
        let input = user_prompt(MAIN_PROMPT)?;
        let mut parts = input.split_whitespace();
        let command = parts.next().unwrap_or("");

        match command {
            CMD_ADD => {
                let mut added_any = false;
                for arg in parts {
                    match i32::from_str(arg) {
                        Ok(num) => {
                            elements.insert(num);
                            added_any = true;
                        }
                        Err(_) => {
                            eprintln!("Invalid input: '{}', please enter an integer.", arg);
                        }
                    }
                }
                if added_any {
                    println!("Elements added.");
                }
            }
            CMD_IDENTITY => match parts.next().and_then(|arg| i32::from_str(arg).ok()) {
                Some(num) => {
                    identity = Some(num);
                    println!("Identity set.");
                }
                None => {
                    eprintln!("Invalid input, please enter an integer.");
                }
            },
            CMD_LIST => {
                println!("Current elements: {:?}", elements);
                if let Some(id) = identity {
                    println!("Identity element: {}", id);
                } else {
                    eprintln!("Identity element not set.");
                }
            }
            CMD_HELP => {
                println!("Available commands:");
                println!(
                    "  {} <element1> <element2> ... - Add elements to the group",
                    CMD_ADD
                );
                println!("  {} <identity> - Set the identity element", CMD_IDENTITY);
                println!("  {} - List current elements and identity", CMD_LIST);
                println!("  {} - Validate and create the group", CMD_CREATE);
                println!("  {} - Exit the program", CMD_EXIT);
            }
            CMD_CREATE => {
                if let Some(identity) = identity {
                    match Group::new(elements.clone(), operation, identity) {
                        Ok(group) => println!("Group created: {:?}", group),
                        Err(e) => eprintln!("Error creating group:\n{}", e),
                    }
                } else {
                    eprintln!("Identity element not set.");
                }
            }
            CMD_EXIT => break,
            _ => println!(
                "Unknown command. Type '{}' for available commands.",
                CMD_HELP
            ),
        }
    }

    Ok(())
}

fn user_prompt(prompt: &str) -> Result<String> {
    println!();
    print!("{}>> ", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    println!();
    Ok(input.trim().to_string())
}
