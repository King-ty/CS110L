use crate::debugger_command::DebuggerCommand;
use crate::dwarf_data::{DwarfData, Error as DwarfError};
use crate::inferior::Inferior;
use crate::inferior::Status;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::collections::HashMap;

pub struct Debugger {
    target: String,
    history_path: String,
    readline: Editor<()>,
    inferior: Option<Inferior>,
    debug_data: DwarfData,
    breakpoints: HashMap<usize, u8>,
}

impl Debugger {
    /// Initializes the debugger.
    pub fn new(target: &str) -> Debugger {
        // DONE (milestone 3): initialize the DwarfData
        let debug_data = match DwarfData::from_file(target) {
            Ok(val) => val,
            Err(DwarfError::ErrorOpeningFile) => {
                println!("Could not open file {}", target);
                std::process::exit(1);
            }
            Err(DwarfError::DwarfFormatError(err)) => {
                println!("Could not debugging symbols from {}: {:?}", target, err);
                std::process::exit(1);
            }
        };
        // debug
        println!("target: {}", target);
        debug_data.print();

        let history_path = format!("{}/.deet_history", std::env::var("HOME").unwrap());
        let mut readline = Editor::<()>::new().expect("Create Editor fail");
        // Attempt to load history from ~/.deet_history if it exists
        let _ = readline.load_history(&history_path);

        Debugger {
            target: target.to_string(),
            history_path,
            readline,
            inferior: None,
            debug_data,
            breakpoints: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.get_next_command() {
                DebuggerCommand::Run(args) => {
                    // Kill existing inferior
                    self.kill().unwrap();
                    if let Some(inferior) =
                        Inferior::new(&self.target, &args, &mut self.breakpoints)
                    {
                        // Create the inferior
                        self.inferior = Some(inferior);
                        // DONE (milestone 1): make the inferior run
                        // You may use self.inferior.as_mut().unwrap() to get a mutable reference
                        // to the Inferior object
                        self.resume();
                    } else {
                        println!("Error starting subprocess");
                    }
                }
                DebuggerCommand::Continue => {
                    if self.inferior.is_some() {
                        self.resume();
                    } else {
                        println!("No running subprocess");
                    }
                }
                DebuggerCommand::Backtrace => {
                    if self.inferior.is_some() {
                        self.inferior
                            .as_mut()
                            .unwrap()
                            .print_backtrace(&self.debug_data)
                            .unwrap();
                    } else {
                        println!("No running subprocess");
                    }
                }
                DebuggerCommand::Quit => {
                    self.kill().unwrap();
                    return;
                }
                DebuggerCommand::Breakpoint(addr_str) => {
                    let breakpoint_addr: usize;
                    if addr_str.starts_with("*") {
                        if let Some(addr) = Self::parse_address(&addr_str[1..]) {
                            breakpoint_addr = addr;
                        } else {
                            println!("Parse address failed");
                            continue;
                        }
                    } else if let Ok(line_num) = usize::from_str_radix(&addr_str, 10) {
                        if let Some(addr) = self.debug_data.get_addr_for_line(None, line_num) {
                            breakpoint_addr = addr;
                        } else {
                            println!("Get address from line failed");
                            continue;
                        }
                    } else if let Some(addr) =
                        self.debug_data.get_addr_for_function(None, &addr_str)
                    {
                        breakpoint_addr = addr;
                    } else {
                        println!("Usage: <b|break|breakpoint> <*address|line|func>");
                        continue;
                    }
                    if self.inferior.is_some() {
                        self.inferior
                            .as_mut()
                            .unwrap()
                            .add_breakpoint(breakpoint_addr, &mut self.breakpoints);
                    } else {
                        self.breakpoints.insert(breakpoint_addr, 0);
                    }
                    println!(
                        "Set breakpoint {} at {:#x}",
                        self.breakpoints.len() - 1,
                        breakpoint_addr
                    );
                }
            }
        }
    }

    fn resume(&mut self) {
        match self
            .inferior
            .as_mut()
            .unwrap()
            .resume(None, &self.breakpoints)
        {
            Ok(status) => match status {
                Status::Stopped(sig, rip) => {
                    // println!("Stopped, signal: {}, size: {}", sig, size);
                    println!("Child stopped (signal {})", sig);
                    let line = match self.debug_data.get_line_from_addr(rip) {
                        Some(line) => format!("{}:{}", line.file, line.number),
                        None => "source file not found".to_string(),
                    };
                    println!("Stopped at {}", line);
                }

                Status::Exited(status) => {
                    println!("Child exited (status {})", status);
                    self.inferior = None;
                }

                Status::Signaled(sig) => {
                    println!("Signaled, signal: {}", sig);
                }
            },
            Err(err) => {
                println!("ERR: {}", err);
            }
        }
    }

    fn kill(&mut self) -> Result<(), nix::Error> {
        if self.inferior.is_some() {
            let inferior = self.inferior.as_mut().unwrap();
            println!("Killing running inferior (pid {})", inferior.pid());
            inferior.kill()?;
            self.inferior = None;
        }
        Ok(())
    }

    fn parse_address(addr: &str) -> Option<usize> {
        let addr_without_0x = if addr.to_lowercase().starts_with("0x") {
            &addr[2..]
        } else {
            &addr
        };
        usize::from_str_radix(addr_without_0x, 16).ok()
    }

    /// This function prompts the user to enter a command, and continues re-prompting until the user
    /// enters a valid command. It uses DebuggerCommand::from_tokens to do the command parsing.
    ///
    /// You don't need to read, understand, or modify this function.
    fn get_next_command(&mut self) -> DebuggerCommand {
        loop {
            // Print prompt and get next line of user input
            match self.readline.readline("(deet) ") {
                Err(ReadlineError::Interrupted) => {
                    // User pressed ctrl+c. We're going to ignore it
                    println!("Type \"quit\" to exit");
                }
                Err(ReadlineError::Eof) => {
                    // User pressed ctrl+d, which is the equivalent of "quit" for our purposes
                    return DebuggerCommand::Quit;
                }
                Err(err) => {
                    panic!("Unexpected I/O error: {:?}", err);
                }
                Ok(line) => {
                    if line.trim().len() == 0 {
                        continue;
                    }
                    self.readline.add_history_entry(line.as_str());
                    if let Err(err) = self.readline.save_history(&self.history_path) {
                        println!(
                            "Warning: failed to save history file at {}: {}",
                            self.history_path, err
                        );
                    }
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    if let Some(cmd) = DebuggerCommand::from_tokens(&tokens) {
                        return cmd;
                    } else {
                        println!("Unrecognized command.");
                    }
                }
            }
        }
    }
}
