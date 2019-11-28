FORCE:

build-%: FORCE
	cargo build --package $*

run-%: FORCE
	cargo run --package $*

generated = $(shell fd -I '^target$$')
clean:
	rm -rf $(generated)
