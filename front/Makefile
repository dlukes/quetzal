.PHONY: all clean

elm_files = $(shell fd '.*\.elm')
target/main.js: $(elm_files)
	elm make $(elm_files) --output $@ --debug

all: target/main.js

generated = $(shell fd -I '^(target|elm-stuff)$$')
clean:
	rm -rf $(generated)
