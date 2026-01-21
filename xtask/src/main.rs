use std::{env, fs::{self, File, OpenOptions, read_dir}, io::{BufRead, BufReader, Seek, SeekFrom, Write}, path::{Path, PathBuf}, process::Command};

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
	mode:                String,
	target_dir:          PathBuf,
	rustsbi_dir:         PathBuf,
	package_dir:         PathBuf,
	workspace_dir:       PathBuf,
	kernel_need_rebuild: bool,
	apps:                Vec<String>,
}

fn hash_dir(dir: &Path) -> anyhow::Result<Vec<u8>> {
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

fn main() -> anyhow::Result<()> {
	let cli = Cli::parse();

	let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	let workspace_dir = package_dir.parent().expect("No workspace dir").to_path_buf();

	let mode = if cli.release() { "release" } else { "debug" }.to_string();

	let mut xtask = Xtask {
		target_dir: workspace_dir.join("target/riscv64gc-unknown-none-elf/").join(&mode),
		mode,
		rustsbi_dir: package_dir.join("rustsbi"),
		package_dir,
		workspace_dir,
		kernel_need_rebuild: false,
		apps: vec![],
	};

	match cli.command {
		Commands::Build { .. } => xtask.build()?,
		Commands::Run { qemu_args, .. } => xtask.run_qemu(qemu_args)?,
	}

	Ok(())
}

impl Xtask {
	fn build(&mut self) -> anyhow::Result<()> {
		fs::create_dir_all(&self.target_dir)?;
		self.build_rustsbi()?;
		self.build_user()?;
		self.build_kernel()?;
		Ok(())
	}

	fn need_rerun(&self, dirs: &[&Path], scope: &str) -> anyhow::Result<bool> {
		let stamp_file = self.target_dir.join(scope).with_extension("stamp");

		let mut combined = Vec::new();
		let changed = match fs::read(&stamp_file) {
			Ok(old) => {
				for dir in dirs {
					combined.extend(hash_dir(dir)?);
				}
				old != combined
			}
			Err(_) => true,
		};

		if !changed {
			println!("✓ {scope} unchanged, skip build");
			return Ok(false);
		}
		fs::write(&stamp_file, combined)?;
		Ok(true)
	}

	fn build_rustsbi(&self) -> anyhow::Result<()> {
		let xtask_dir = self.rustsbi_dir.join("xtask");
		let proto_dir = self.rustsbi_dir.join("prototyper");
		if !self.need_rerun(&[&xtask_dir, &proto_dir], "rustsbi")? {
			return Ok(());
		}

		if !xtask_dir.exists() || !proto_dir.exists() {
			anyhow::bail!("rustsbi submodule not initialized correctly");
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

		println!("✓ RustSBI build successful");
		Ok(())
	}

	fn build_kernel(&self) -> anyhow::Result<()> {
		println!("Building kernel...");

		let linker_script = self.package_dir.join("linker-kernel.ld");
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

	fn build_user(&mut self) -> anyhow::Result<()> {
		println!("Building user...");

		let linker_script = self.package_dir.join("linker-user.ld");

		let linker_script_abs = std::fs::canonicalize(&linker_script)?;

		let rustflags = format!("-C link-arg=-T{} -C force-frame-pointers=yes", linker_script_abs.display());

		let mut apps: Vec<_> = read_dir(self.workspace_dir.join("user/src/bin"))?
			.map(|entry| {
				let mut name_with_ext = entry.unwrap().file_name().into_string().unwrap();
				// remove extension
				name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
				name_with_ext
			})
			.collect();
		apps.sort();
		self.apps = apps;

		let file = OpenOptions::new().read(true).write(true).open(&linker_script)?;
		let mut reader = BufReader::new(&file);
		let mut offset = 0;
		let mut line = String::new();
		let mut writer = &file;
		let base_address: u64 = 0x80400000;
		let step: u64 = 0x20000;

		loop {
			line.clear();
			let bytes = reader.read_line(&mut line)?;
			if bytes == 0 {
				break;
			}
			if line.trim_start().starts_with("BASE_ADDRESS") {
				break;
			}
			offset += bytes as u64;
		}

		for (id, bin) in self.apps.iter().enumerate() {
			println!("Building {bin}...");

			writer.seek(SeekFrom::Start(offset))?;
			let new_address = base_address + id as u64 * step;
			let new_line = format!("BASE_ADDRESS = 0x{new_address:08x};\n");
			if line.len().ne(&new_line.len()) {
				anyhow::bail!("new line size must equals to the old one");
			}
			writer.write_all(new_line.as_bytes())?;
			writer.flush()?;
			let mut command = Command::new("cargo");
			command.args(["build", "--bin", bin]);
			if self.mode.eq("release") {
				command.arg("--release");
			}
			let status =
				command.args(["--target", "riscv64gc-unknown-none-elf"]).env("RUSTFLAGS", &rustflags).status()?;

			if !status.success() {
				anyhow::bail!("User build failed");
			}

			println!("Application {bin} start with address 0x{new_address:08x}");
		}

		// restore the base address
		writer.seek(SeekFrom::Start(offset))?;
		let new_line = format!("BASE_ADDRESS = 0x{base_address:08x};\n");
		writer.write_all(new_line.as_bytes())?;
		writer.flush()?;

		println!("✓ User build successful");
		self.generate_user_app_data()
	}

	fn run_qemu(&mut self, extra_qemu_args: Vec<String>) -> anyhow::Result<()> {
		self.build()?;

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

	/// Generate app binaries linker.
	fn generate_user_app_data(&mut self) -> anyhow::Result<()> {
		if !self.need_rerun(&[&self.workspace_dir.join("user/src")], "user")? {
			return Ok(());
		}
		let app_link_file_path = self.workspace_dir.join("kernel/src/asm/link_app.S");
		let mut app_link_file = File::create(&app_link_file_path)?;

		writeln!(
			app_link_file,
			r#"/* Generated by build.rs */
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad {}"#,
			self.apps.len()
		)?;

		for i in 0..self.apps.len() {
			writeln!(app_link_file, r#"    .quad app_{}_start"#, i)?;
		}
		writeln!(app_link_file, r#"    .quad app_{}_end"#, self.apps.len() - 1)?;

		for (idx, app) in self.apps.iter().enumerate() {
			let app_binary = self.target_dir.join(app);
			let aapp_bin_binary = app_binary.with_extension("bin");

			if !app_binary.exists() {
				anyhow::bail!("App binary not found at {}.", app_binary.display());
			}

			let status = Command::new("rust-objcopy")
				.arg("--strip-all")
				.arg(&app_binary)
				.arg("-O")
				.arg("binary")
				.arg(&aapp_bin_binary)
				.status()?;

			if !status.success() {
				anyhow::bail!("{} rust-objcopy failed with status: {:?}", app_binary.display(), status);
			}

			writeln!(
				app_link_file,
				r#"
    .section .data
    .global app_{0}_start
    .global app_{0}_end
app_{0}_start:
    .incbin "{1}.bin"
app_{0}_end:"#,
				idx,
				self.target_dir.join(app).display()
			)?;
		}
		println!("Generated {}", app_link_file_path.display());
		// We do not realize elf yet, we need to clean the kernel, because it include
		// the last user bins, it will not be recompiled
		let status = Command::new("cargo").args(["clean", "--package", "kernel"]).status()?;

		if !status.success() {
			anyhow::bail!("Clean kernel failed");
		}

		self.kernel_need_rebuild = true;
		Ok(())
	}
}
