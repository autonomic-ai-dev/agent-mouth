.PHONY: all build test check clean install

all: build

build:
	cargo build --release -p agent-mouth

test:
	cargo test --release -p agent-mouth

check:
	cargo check -p agent-mouth

clean:
	cargo clean -p agent-mouth

install: build
	cp target/release/agent-mouth ~/.local/bin/agent-mouth
