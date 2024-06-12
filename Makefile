shell := /bin/bash

INSTALL_DIR=/usr/local/bin

# Check if synless is already installed in the current path
ifneq ("$(wildcard $(shell which synless))","")
EXISTING_BIN_PATH = $(shell which synless)
endif


# Build synless executable
./target/debug/synless:
	cargo build


# Install the executable to the specified INSTALL_DIR
.PHONY: install
install: ./target/debug/synless
ifdef EXISTING_BIN_PATH
	$(error ERROR: Conflicting binary already installed in the current path at $(EXISTING_BIN_PATH))
endif
	sudo cp $(shell pwd)/target/debug/synless $(INSTALL_DIR)


# Uninstall the executable from EXISTING_BIN_PATH
.PHONY: uninstall
uninstall: $(EXISTING_BIN_PATH)
ifndef EXISTING_BIN_PATH
	$(error ERROR: synless binary not installed in the current path)
endif
	sudo rm -i $(EXISTING_BIN_PATH)


# Remove generated files
.PHONY: clean
clean:
	rm -r target
