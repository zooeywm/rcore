use core::arch::global_asm;

use riscv::{interrupt::{Trap, supervisor::{Exception, Interrupt}}, register::{scause, sie, stval, stvec::{self, Stvec, TrapMode}}};

use crate::{error, syscall::syscall, system::set_next_trigger, task::{exit_current_and_run_next, suspend_current_and_run_next}, trap::context::TrapContext};

pub mod context;

global_asm!(include_str!("trap.S"));

/// Init trap with set stvec to Direct mode
pub fn init() {
	unsafe extern "C" {
		fn __alltraps();
	}

	unsafe { stvec::write(Stvec::new(__alltraps as *const () as usize, TrapMode::Direct)) }
}

pub fn enable_timer_interrupt() {
	unsafe {
		sie::set_stimer();
	}
}

#[unsafe(no_mangle)]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
	let scause = scause::read();
	let stval = stval::read();

	match scause.cause().try_into::<Interrupt, Exception>().expect("Wrong trap type") {
		Trap::Exception(Exception::UserEnvCall) => {
			cx.sepc += 4;
			// a7 - syscall ID, a0~a2: args, a0: also record return value
			cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
		}
		Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
			error!(
				"PageFault in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.",
				stval, cx.sepc
			);
			exit_current_and_run_next();
		}
		Trap::Exception(Exception::IllegalInstruction) => {
			error!("IllegalInstruction in application, kernel killed it.");
			exit_current_and_run_next();
		}
		Trap::Exception(e) => {
			error!("{e:?} in application, kernel killed it.");
			exit_current_and_run_next();
		}
		Trap::Interrupt(Interrupt::SupervisorTimer) => {
			set_next_trigger();
			suspend_current_and_run_next();
		}
		_ => {
			panic!("Unsupported trap {:#?}, stval = {:#x}!", scause.cause(), stval)
		}
	}
	cx
}
