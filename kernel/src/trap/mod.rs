use core::arch::global_asm;

use riscv::{interrupt::{Trap, supervisor::{Exception, Interrupt}}, register::{scause, stval, stvec::{self, Stvec, TrapMode}}};

use crate::{batch::run_next_app, error, syscall::syscall, trap::context::TrapContext};

pub mod context;

global_asm!(include_str!("trap.S"));

/// Init trap with set stvec to Direct mode
pub fn init() {
	unsafe extern "C" {
		fn __alltraps();
	}

	unsafe { stvec::write(Stvec::new(__alltraps as *const () as usize, TrapMode::Direct)) }
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
		Trap::Exception(e) => {
			error!("{e:?} in application, kernel killed it.");
			run_next_app();
		}
		_ => {
			panic!("Unsupported trap {:#?}, stval = {:#x}!", scause.cause(), stval)
		}
	}
	cx
}
