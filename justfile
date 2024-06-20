# justfile


[private]
default:
    @just --list

alias b := build
alias r := run
alias c := clippy

# targets := ["esp32", "esp32c3"]

[group('cargo')]
build board="esp32c3":
    cargo +esp build --target {{ if board == "esp32" { "xtensa-esp32-none-elf" } else { "riscv32imc-unknown-none-elf" } }} --features {{board}} --release

[group('cargo')]
run board="esp32c3":
    cargo +esp run --target {{ if board == "esp32" { "xtensa-esp32-none-elf" } else { "riscv32imc-unknown-none-elf" } }} --features {{ board }} --release

[group('cargo')]
clippy board="esp32c3":
    cargo +esp clippy --target {{ if board == "esp32" { "xtensa-esp32-none-elf" } else { "riscv32imc-unknown-none-elf" } }} --features {{ board }} --release

# test board: fmt
#     cargo +esp nextest run --target {{ if board == "esp32" { "xtensa-esp32-none-elf" } else { "riscv32imc-unknown-none-elf" } }} --features {{ board }} --release

[group('ci')]
prepare: fmt (_prepare "esp32") (_prepare "esp32c3")

[group('ci')]
fix board:
    cargo +esp fix --target {{ if board == "esp32" { "xtensa-esp32-none-elf" } else { "riscv32imc-unknown-none-elf" } }} --features {{ board }} --release --allow-dirty

[group('ci')]
fmt: _taplo
    cargo +nightly fmt -- --config-path ./rustfmt.nightly.toml

_taplo: 
    @taplo fmt

[group('ci')]
_ci_fmt:
    cargo +nightly fmt --all -- --config-path ./rustfmt.nightly.toml --check --color always

_ci_build board: (build board)

[group('ci')]
_ci_clippy board:
    cargo +esp clippy --target {{ if board == "esp32" { "xtensa-esp32-none-elf" } else { "riscv32imc-unknown-none-elf" } }} --features {{ board }} --release --workspace -- -D warnings

_prepare board: (clippy board) (build board)
