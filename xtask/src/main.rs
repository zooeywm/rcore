use std::{env, fs, path::{Path, PathBuf}, process::Command};

use anyhow::Result;
use clap::{Parser, Subcommand};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Build automation tasks", long_about = None)]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

impl Cli {
	fn release(&self) -> bool {
		match self.command {
			Commands::Build { release } => release,
			Commands::Run { qemu_args: _, release } => release,
		}
	}
}

#[derive(Subcommand)]
enum Commands {
	Build {
		#[arg(long)]
		release: bool,
	},
	Run {
		#[arg(trailing_var_arg = true, allow_hyphen_values = true)]
		qemu_args: Vec<String>,
		#[arg(long)]
		release:   bool,
	},
}

struct Xtask {
	mode:        String,
	target_dir:  PathBuf,
	rustsbi_dir: PathBuf,
}

fn hash_dir(dir: &Path) -> Result<Vec<u8>> {
	let mut hasher = Sha256::new();

	for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
		if entry.file_type().is_file() {
			let path = entry.path();
			let data = fs::read(path)?;
			hasher.update(path.to_string_lossy().as_bytes());
			hasher.update(&data);
		}
	}

	Ok(hasher.finalize().to_vec())
}

fn main() -> Result<()> {
	let cli = Cli::parse();

	let package_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));

	let mode = if cli.release() { "release" } else { "debug" }.to_string();
	let xtask = Xtask {
		target_dir: package_root
			.parent()
			.expect("No workspace dir")
			.join("target/riscv64gc-unknown-none-elf/")
			.join(&mode),
		mode,
		rustsbi_dir: package_root.join("rustsbi"),
	};

	match cli.command {
		Commands::Build { .. } => xtask.build()?,
		Commands::Run { qemu_args, .. } => xtask.run_qemu(qemu_args)?,
	}

	Ok(())
}

impl Xtask {
	fn build(self) -> Result<()> {
		self.build_rustsbi()?;
		self.build_kernel()?;

		Ok(())
	}

	fn build_rustsbi(&self) -> Result<()> {
		println!("Checking RustSBI changes...");

		let xtask_dir = self.rustsbi_dir.join("xtask");
		let proto_dir = self.rustsbi_dir.join("prototyper");
		let stamp_file = self.rustsbi_dir.join("target/riscv64gc-unknown-none-elf/release/.rustsbi.stamp");

		if !xtask_dir.exists() || !proto_dir.exists() {
			anyhow::bail!("rustsbi submodule not initialized correctly");
		}
		let mut combined = Vec::new();
		combined.extend(hash_dir(&xtask_dir)?);
		combined.extend(hash_dir(&proto_dir)?);

		let changed = match fs::read(&stamp_file) {
			Ok(old) => old != combined,
			Err(_) => true,
		};

		if !changed {
			println!("✓ RustSBI unchanged, skip build");
			return Ok(());
		}

		println!("Building RustSBI...");

		if !self.rustsbi_dir.exists() {
			anyhow::bail!("rustsbi submodule not initialized. Run 'git submodule update --init --recursive'");
		}

		let status =
			Command::new("cargo").current_dir(&self.rustsbi_dir).args(["prototyper", "-f", "jump"]).status()?;

		if !status.success() {
			anyhow::bail!("RustSBI build failed");
		}

		fs::write(&stamp_file, combined)?;
		println!("✓ RustSBI build successful");
		Ok(())
	}

	fn build_kernel(&self) -> Result<()> {
		println!("Building kernel...");

		let linker_script = PathBuf::from("/home/zooeywm/repos/mine/rcore/xtask/linker.ld");
		let linker_script_abs = std::fs::canonicalize(&linker_script)?;

		let rustflags = format!("-C link-arg=-T{} -C force-frame-pointers=yes", linker_script_abs.display());

		let mut command = Command::new("cargo");
		command.args(["build", "--bin", "kernel"]);
		if self.mode.eq("release") {
			command.arg("--release");
		}
		let status =
			command.args(["--target", "riscv64gc-unknown-none-elf"]).env("RUSTFLAGS", &rustflags).status()?;

		if !status.success() {
			anyhow::bail!("Kernel build failed");
		}

		println!("✓ Kernel build successful");
		let kernel_binary = self.target_dir.join("kernel");
		let kernel_bin_binary = self.target_dir.join("kernel.bin");

		if !kernel_binary.exists() {
			anyhow::bail!(
				"Kernel binary not found at {}. Run 'cargo build --release' first.",
				kernel_binary.display()
			);
		}

		let status = Command::new("rust-objcopy")
			.arg("--strip-all")
			.arg(kernel_binary)
			.arg("-O")
			.arg("binary")
			.arg(&kernel_bin_binary)
			.status()?;

		if !status.success() {
			anyhow::bail!("rust-objcopy failed with status: {:?}", status);
		}

		println!("✓ Generated kernel raw binary: {}", kernel_bin_binary.display());
		Ok(())
	}

	fn run_qemu(&self, extra_qemu_args: Vec<String>) -> Result<()> {
		self.build_kernel()?;

		let bios_path = self.rustsbi_dir.join("target/riscv64gc-unknown-none-elf/release/rustsbi-prototyper.bin");
		let kernel_bin_binary = self.target_dir.join("kernel.bin");

		if !bios_path.exists() {
			anyhow::bail!(
				"RustSBI binary not found at {}. Run 'cargo build --release' in kernel directory first.",
				bios_path.display()
			);
		}

		println!("Running QEMU...");

		let _bios_path_str = bios_path.to_string_lossy();
		let _kernel_raw_binary_str = kernel_bin_binary.to_string_lossy();

		let mut cmd = Command::new("qemu-system-riscv64");
		cmd.arg("-machine").arg("virt");
		cmd.arg("-nographic");
		cmd.arg("-bios").arg(&bios_path);
		cmd.arg("-device").arg(format!("loader,file={},addr=0x80200000", kernel_bin_binary.display()));

		if !extra_qemu_args.is_empty() {
			cmd.args(&extra_qemu_args);
		}

		let status = cmd.status()?;

		std::process::exit(status.code().unwrap_or(1));
	}
}
