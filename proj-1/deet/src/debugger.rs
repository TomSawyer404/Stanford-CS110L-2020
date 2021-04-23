use crate::debugger_command::DebuggerCommand;
use crate::inferior::Inferior;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use crate::dwarf_data::{DwarfData, Error as DwarfError}; // for milestone3
use crate::inferior::Restorepoint;  // for milestone6
use std::collections::HashMap;      // for milestone6

pub struct Debugger {
    target: String,
    history_path: String,
    readline: Editor<()>,
    inferior: Option<Inferior>,
    debug_data: DwarfData,      // for milestone3
    break_list: Vec<usize>,     // for milestone5
    restore_map: HashMap<usize, Restorepoint>,  // for milestone6
}

impl Debugger {
    /// Initializes the debugger.
    pub fn new(target: &str) -> Debugger {
        // TODO (milestone 3): initialize the DwarfData
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
        //debug_data.print(); // for milestone6

        let history_path = format!("{}/.deet_history", std::env::var("HOME").unwrap());
        let mut readline = Editor::<()>::new();
        // Attempt to load history from ~/.deet_history if it exists
        let _ = readline.load_history(&history_path);

        let break_list = Vec::new();        // for milestone5
        let restore_map = HashMap::new();   // for milestone6
        Debugger {
            target: target.to_string(),
            history_path,
            readline,
            inferior: None,
            debug_data,     // for milestone3
            break_list,     // for milestone5
            restore_map,    // for milestone6
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.get_next_command() {
                DebuggerCommand::Run(args) => {
                    // for milestone2
                    // kill previous tracee before we run a new tracee
                    let obj = self.inferior.as_mut();
                    if let Some(i) = obj {
                        println!("Killing running inferior (pid {})", i.pid());
                        i.kill_myself().unwrap(); 
                    }
                    
                    if let Some(inferior) = Inferior::new(&self.target, &args) {
                        // Create the inferior
                        self.inferior = Some(inferior);
                        
                        // milestone5
                        let tracee = self.inferior.as_mut().unwrap();
                        tracee.set_breakpoint(
                            &self.break_list, 
                            &mut self.restore_map  // milestone6
                        ).unwrap();
                        self.break_list.clear();

                        // TODO (milestone 1): make the inferior run
                        // You may use self.inferior.as_mut().unwrap() to get a mutable reference
                        // to the Inferior object
                        if let Ok(status) = tracee.wake_up(&self.restore_map) {
                            use crate::inferior::Status;
                            match status {
                                Status::Exited(exit_code) => {
                                    println!("Child exited (status {})", exit_code);
                                },
                                Status::Signaled(signal) => {
                                    println!("Child got a signal ({})", signal);
                                },
                                Status::Stopped(signal, rip) => {
                                    println!("Child stopped with signal: {}", signal);
                                    let line = DwarfData::get_line_from_addr(&self.debug_data, rip);
                                    if let Some(i) = line {
                                        println!("Stopped at: {}", i);
                                    }
                                }
                            }
                        }
                    } else {
                        println!("Error starting subprocess");
                    }
                }
                DebuggerCommand::Quit => {
                    // Kill remaining tracee before exit
                    let obj = self.inferior.as_mut();
                    if let Some(i) = obj {
                        if i.try_wait() {
                            return;
                        }
                        println!("Killing running inferior (pid {})", i.pid());
                        i.kill_myself().unwrap(); 
                    }
                    return;
                }
                DebuggerCommand::Continue => {
                    let obj = self.inferior.as_mut();
                    if let None = obj {
                        eprintln!("You need to run a tracee first!");
                    } else {
                        let tracee = obj.unwrap();
                        if let Ok(status) = tracee.wake_up(&self.restore_map) {
                            use crate::inferior::Status;
                            match status {
                                Status::Exited(exit_code) => {
                                    println!("Continuing... Child exited (status {})", exit_code);
                                },
                                Status::Signaled(signal) => {
                                    println!("Continuing... Child got a signal ({})", signal);
                                },
                                Status::Stopped(signal, rip) => {
                                    println!("Continuing... Child stopped with signal: {}", signal);
                                    let line = DwarfData::get_line_from_addr(&self.debug_data, rip);
                                    if let Some(i) = line {
                                        println!("Stopped at: {}", i);
                                        println!("%rip = {:#x}", rip);
                                    }
                                },
                            }
                        }
                        
                        if !self.break_list.is_empty() {
                            tracee.set_breakpoint(
                                &self.break_list, 
                                &mut self.restore_map
                            ).unwrap();
                            self.break_list.clear();
                        }
                    }
                }
                DebuggerCommand::Backtrace => {
                    let obj = self.inferior.as_mut().unwrap();
                    obj.print_backtrace(&self.debug_data).unwrap();
                }
                DebuggerCommand::Break(br_arg) => {
                    // Check if first char is '*'
                    let addr_without_0x = if br_arg[..]
                        .to_lowercase()
                        .starts_with("*0x") 
                    {
                        &br_arg[3..]
                    } else {
                        // for milestone7
                        let br_addr: Option<usize>;
                        if let Ok(i) = br_arg.parse::<usize>() {
                            br_addr = DwarfData::get_addr_for_line(
                                &self.debug_data, 
                                None, 
                                i
                            );
                        } else {
                            br_addr = DwarfData::get_addr_for_function(
                                &self.debug_data, 
                                None, 
                                &br_arg[..]
                            );
                        }
                        if let None = br_addr {
                            eprintln!("Invalid breakpoint!");
                            continue;
                        }
                        self.break_list.push( br_addr.unwrap() );
                        println!("Set a breakpoint at {:#x}", 
                                 self.break_list.last().unwrap());
                        continue;
                    };
                    let br_addr = usize::from_str_radix(addr_without_0x, 16).unwrap();
                    self.break_list.push( br_addr );
                    println!("Set a breakpoint at {:#x}", 
                             self.break_list.last().unwrap());
                    
                }
            }
        }
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
