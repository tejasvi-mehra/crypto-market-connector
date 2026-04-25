-include .env.local

export 

run: 
	cargo run

run-dev:
	cargo watch -x run