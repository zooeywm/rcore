use std::{env, path::PathBuf, process::Command};

fn main() {
	let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
	let rustsbi_dir = manifest_dir.join("rustsbi");

	assert!(rustsbi_dir.exists(), "rustsbi submodule not initialized");

	// use cargo build -vv to view this log
	println!("building rustsbi firmware");
	let status = Command::new("cargo")
		.current_dir(&rustsbi_dir)
		.env_remove("RUSTFLAGS")
		.env_remove("CARGO_ENCODED_RUSTFLAGS")
		.env_remove("RUSTC_WORKSPACE_WRAPPER")
		.args([
			"run",
			"--target",
			"x86_64-unknown-linux-gnu",
			"--package",
			"xtask",
			"--release",
			"--",
			"prototyper",
			"-f",
			"jump",
		])
		.status()
		.expect("failed to build rustsbi-prototyper");

	assert!(status.success());

	println!("cargo:rerun-if-changed=rustsbi/library");
	println!("cargo:rerun-if-changed=rustsbi/prototyper");
	println!("cargo:rerun-if-changed=rustsbi/xtask");
}
