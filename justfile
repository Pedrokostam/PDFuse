alias bfuse := build-pdfuse
alias ball := build-all
alias bhaal := build-all
alias bfind := build-pdfind
alias rfuse := run-pdfuse
alias rfind := run-pdfind

# List all available recipes
default:
   just --list

[group('build')]
build-pdfuse:
   cargo build --bin pdfuse

[group('build')]
build-pdfind:
   cargo build --bin pdfind

[group('build')]
build-all: build-pdfind build-pdfuse

[group('run')]
run-pdfuse:
   cargo run --bin pdfuse

[group('run')]
run-pdfind:
   cargo run --bin pdfind

# Launch all tests in the whole workspace
test:
   cargo test --no-run -p pdfuse-utils
   cargo test --no-run -p pdfuse-sizing
   cargo test --no-run -p pdfuse-parameters
   cargo test --no-run -p pdfuse-commandline
   cargo test --no-run -p pdfuse-merging
   cargo test -q --message-format short -p pdfuse-utils 2>/dev/null
   cargo test -q --message-format short -p pdfuse-sizing 2>/dev/null
   cargo test -q --message-format short -p pdfuse-parameters 2>/dev/null
   cargo test -q --message-format short -p pdfuse-commandline 2>/dev/null
   cargo test -q --message-format short -p pdfuse-merging 2>/dev/null
