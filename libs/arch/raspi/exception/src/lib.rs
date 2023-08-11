#![no_std]

use core::{
    arch::{asm, global_asm},
    fmt::Display,
};

global_asm!(include_str!("context.S"));
global_asm!(include_str!("vector.S"));
global_asm!(include_str!("trampoline.S"));

#[repr(C)]
struct Cpu_Context {
    gpr: [u64; 31],
    sp: u64,
    fpr: [f64; 32],
    esr_el1: u64,
    elr_el1: u64,
    spsr_el1: u64,
    lr: u64,
    far_el1: u64,
    unused: u64,
}

pub fn install_exception_handlers() -> u64 {
    let mut addr: u64 = 0;
    unsafe {
        asm!("adr {addr}, exception_vectors", "msr vbar_el1, {addr}", addr = inout(reg) addr);
    }
    addr
}

impl Display for Cpu_Context {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Exception Syndrome: {:#x}", self.esr_el1)?;
        writeln!(f, "Faulting Address: {:#x}", self.far_el1)?;
        writeln!(f, "Saved Program Status: {:#x}", self.spsr_el1)?;
        writeln!(f, "Exception Link Register: {:#x}", self.elr_el1)?;
        writeln!(f, "Link Register: {:#x}", self.lr)?;
        writeln!(f, "Stack Pointer: {:#x}", self.sp)?;
        writeln!(f, "")?;
        writeln!(f, "General Purpose Registers:")?;
        for i in (0..30).step_by(2) {
            writeln!(
                f,
                "X{:02}: {:#018x}  X{:02}: {:#018x}",
                i,
                self.gpr[i],
                i + 1,
                self.gpr[i + 1]
            )?;
        }
        writeln!(f, "X30: {:#018x}", self.gpr[30])?;

        writeln!(f, " ")?;
        for i in (0..32).step_by(2) {
            writeln!(
                f,
                "Q{:02}: {:#018x}  Q{:02}: {:#018x}",
                i,
                self.fpr[i] as u64,
                i + 1,
                self.fpr[i + 1] as u64
            )?;
        }

        Ok(())
    }
}

#[no_mangle]
extern "C" fn current_elx_synchronous(context: Cpu_Context) {
    panic!("Uncaught exception! Dumping CPU State: \n\n{}", context);
}

#[no_mangle]
extern "C" fn current_elx_irq(context: Cpu_Context) {
    panic!("Uncaught exception! Dumping CPU State: \n{}", context);
}

#[no_mangle]
extern "C" fn current_elx_fiq(context: Cpu_Context) {
    panic!("Uncaught exception! Dumping CPU State: \n{}", context);
}

#[no_mangle]
extern "C" fn current_elx_serror(context: Cpu_Context) {
    panic!("Uncaught exception! Dumping CPU State: \n{}", context);
}
