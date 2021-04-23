use nix::sys::ptrace;
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use std::process::Child;
use crate::dwarf_data::{DwarfData}; // for milestone3
use std::collections::HashMap;      // for milestone6
use nix::sys::signal::Signal;       // for milestone6

#[derive(Clone)]
pub struct Restorepoint {
    addr: usize,
    orig_byte: u8,
}

pub enum Status {
    /// Indicates inferior stopped. Contains the signal that stopped the process, as well as the
    /// current instruction pointer that it is stopped at.
    Stopped(signal::Signal, usize),

    /// Indicates inferior exited normally. Contains the exit status code.
    Exited(i32),

    /// Indicates the inferior exited due to a signal. Contains the signal that killed the
    /// process.
    Signaled(signal::Signal),
}

/// This function calls ptrace with PTRACE_TRACEME to enable debugging on a process. You should use
/// pre_exec with Command to call this in the child process.
fn child_traceme() -> Result<(), std::io::Error> {
    ptrace::traceme().or(Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "ptrace TRACEME failed",
    )))
}

pub struct Inferior {
    child: Child,
}

impl Inferior {
    /// Attempts to start a new inferior process. Returns Some(Inferior) if successful, or None if
    /// an error is encountered.
    pub fn new(target: &str, args: &Vec<String>) -> Option<Inferior> {
        // TODO: implement me!
        use std::process::Command;
        let mut child_cmd = Command::new( target );
        let child_cmd = child_cmd.args( args );
        unsafe {
            use std::os::unix::process::CommandExt;
            child_cmd.pre_exec( child_traceme );
        }
        
        let child_ps: Child = child_cmd
            .spawn()
            .expect("Failed to spawn a child!");
        
        let ret_obj: Inferior = Inferior { child: child_ps };
        loop {
            match ret_obj.wait(None) {
                Ok(_i) => {
                    return Some(ret_obj);
                },
                Err(_) => break,
            }
        }
        
        None
    }

    pub fn wake_up(&mut self, rs_map: &HashMap<usize, Restorepoint>) 
        -> Result<Status, nix::Error> {
        // In milestone1, you just return Ok(status)
        ptrace::cont(self.pid(), None)?;
        let status = self.wait(None)?;

        // For milestone6
        if let Status::Stopped(signal, rip) = status {
            if !rs_map.is_empty() { // probabily stopped by SIGINT
                if let Signal::SIGTRAP = signal {
                    if let Some(i) = rs_map.get( &(rip - 1) ) {
                        println!("breakpoint at {:#x}", rip-1);
                        
                        self.write_byte(i.addr, i.orig_byte)?;
                    } 
                }
            }
        }
        
        Ok(status)
    }

    pub fn kill_myself(&mut self) -> Result<(), std::io::Error> {
        // For milestone2
        self.child.kill()
    }
    
    pub fn try_wait(&mut self) -> bool {
        // For milestone2
        // Attempts to collect the exit status of the child.
        // We got an Err(_e) because the child has already
        //   exited, so try_wait() couldn't find that process.
        if let Err( _e ) = self.child.try_wait() {
            return true;
        }
        false
    }

    pub fn print_backtrace(&self, debug_data: &DwarfData) -> Result<(), nix::Error> {
        // For milestone3
        
        // Get regs value
        let regs_st = ptrace::getregs( self.pid() ).unwrap();
        let mut rip: usize = regs_st.rip as usize;
        let mut rbp: usize = regs_st.rbp as usize;
        //println!("%rip = {:#x}", rip);

        // Print filename and linenumber
        loop {
            let line = DwarfData::get_line_from_addr(debug_data, rip).unwrap();
            let func_name = DwarfData::get_function_from_addr(debug_data, rip).unwrap();
            println!("{} ({})", func_name, line);
        
            if func_name == "main" {
                break;
            }
            rbp = rbp + 8;
            rip = ptrace::read(self.pid(), rbp as ptrace::AddressType)? as usize;
            rbp = rbp - 8;
            rbp = ptrace::read(self.pid(), rbp as ptrace::AddressType)? as usize;
        }
        
        Ok(())
    }

    fn align_addr_to_word(&self, addr: usize) -> usize {
        // for milestone5
        use std::mem::size_of;
        addr & (-(size_of::<usize>() as isize) as usize)
    }

    pub fn set_breakpoint(
        &self, br_list: &Vec<usize>, 
        rs_map: &mut HashMap<usize, Restorepoint>
    ) -> Result<u8, nix::Error> {
        // for milestone5, 6
        let val = 0xcc; // INT instruction in x64
        for i in br_list {
            let addr: usize = *i;

            let aligned_addr = self.align_addr_to_word(addr);
            let byte_offset = addr - aligned_addr;
            let word = ptrace::read(self.pid(), aligned_addr as ptrace::AddressType)? as u64;
            
            let orig_byte = (word >> 8 * byte_offset) & 0xff;
            let restore_st: Restorepoint = Restorepoint { 
                addr: addr,
                orig_byte: orig_byte as u8,
            };
            rs_map.insert(addr, restore_st);
            
            let masked_word = word & !(0xff << 8 * byte_offset);
            let updated_word = masked_word | ((val as u64) << 8 * byte_offset);
            ptrace::write(
                self.pid(),
                aligned_addr as ptrace::AddressType,
                updated_word as *mut std::ffi::c_void,
                )?;
        }
        Ok(0)
    }

    fn write_byte(&mut self, addr: usize, val: u8) -> Result<u8, nix::Error> {
        // for milestone6
        let aligned_addr = self.align_addr_to_word(addr);
        let byte_offset = addr - aligned_addr;
        let word = ptrace::read(self.pid(), aligned_addr as ptrace::AddressType)? as u64;
        let orig_byte = (word >> 8 * byte_offset) & 0xff;
        let masked_word = word & !(0xff << 8 * byte_offset);
        let updated_word = masked_word | ((val as u64) << 8 * byte_offset);
        ptrace::write(
            self.pid(),
            aligned_addr as ptrace::AddressType,
            updated_word as *mut std::ffi::c_void,
        )?;
        Ok(orig_byte as u8)
    }

    /// Returns the pid of this inferior.
    pub fn pid(&self) -> Pid {
        nix::unistd::Pid::from_raw(self.child.id() as i32)
    }

    /// Calls waitpid on this inferior and returns a Status to indicate the state of the process
    /// after the waitpid call.
    pub fn wait(&self, options: Option<WaitPidFlag>) -> Result<Status, nix::Error> {
        Ok(match waitpid(self.pid(), options)? {
            WaitStatus::Exited(_pid, exit_code) => Status::Exited(exit_code),
            WaitStatus::Signaled(_pid, signal, _core_dumped) => Status::Signaled(signal),
            WaitStatus::Stopped(_pid, signal) => {
                let regs = ptrace::getregs(self.pid())?;
                Status::Stopped(signal, regs.rip as usize)
            }
            other => panic!("waitpid returned unexpected status: {:?}", other),
        })
    }
}
