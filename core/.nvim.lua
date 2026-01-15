vim.g.project_config = {
    rust_analyzer = {
        settings = {
            ["rust-analyzer"] = {
                check = {
                    command = "clippy", -- 使用 clippy
                    allTargets = false, -- 只检查当前 crate
                    extraArgs = { "--target", "riscv64gc-unknown-none-elf" }, -- 指定 target
                    targets = { "riscv64gc-unknown-none-elf" },
                },
                cargo = {
                    target = "riscv64gc-unknown-none-elf",
                },
            },
        },
    },
}
