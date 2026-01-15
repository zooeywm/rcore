    # Show that we want to push the entire content to a section named .text.entry.
    .section .text.entry
    # Tell the compiler that _start is a global symbol, can be used by other object file.
    .globl _start
# Declare a symbol named _start, means the address of _start is the address of la instruction.
_start:
    # Before the control is transferred to Rust entry point, set the stack pointer to the top of stack.
    la sp, boot_stack_top
    call rust_main

# We use bss.stack as stack area. Note that on RISCV, the stack addr is increase from higher to lower.
    .section .bss.stack
    # Define global stack bottom addr
    .globl boot_stack_lower_bound
boot_stack_lower_bound:
    # The max addr size is 64KiB
    .space 4096 * 16
    # Define global stack top addr
    .globl boot_stack_top
boot_stack_top:
