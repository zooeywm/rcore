use riscv::register::sstatus::{self, SPP, Sstatus};

/// Resources need to be store when trap is triggered.
#[repr(C)]
pub struct TrapContext {
	/// 32 general registers
	pub x:       [usize; 32],
	/// sstatus CSR
	pub sstatus: Sstatus,
	/// return address after trap
	pub sepc:    usize,
}

impl TrapContext {
	/// set stack pointer to x2 reg (sp)
	pub fn set_sp(&mut self, sp: usize) { self.x[2] = sp; }

	/// init app context
	pub fn app_init_context(entry: usize, sp: usize) -> Self {
		let mut sstatus = sstatus::read(); // CSR sstatus
		sstatus.set_spp(SPP::User); //previous privilege mode: user mode
		let mut cx = Self {
			x: [0; 32],
			sstatus,
			sepc: entry, // entry point of app
		};
		cx.set_sp(sp); // app's user stack pointer
		cx // return initial Trap Context of app
	}
}
