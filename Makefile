# I made this shit compatible with Windows, Linux and macOS, yay :D

# POSIX-friendly Makefile for building the Cargo workspace and packaging plugins
# Builds the entire workspace with `cargo build --release` and then discovers
# API plugins under `api/*/*`, reads their `Cargo.toml` package name and copies
# the produced dynamic library into `target/release/api/<api>/<version>/`.

TARGET_DIR := target/release

.PHONY: all build setup_structure clean

all: build setup_structure
	@echo "✨ [Clock] Full workspace build complete"

build:
	@echo "🛠️  Building Cargo workspace (release)..."
	@cargo build --release

setup_structure:
	@echo "📂 Organizing plugin artifacts into $(TARGET_DIR)/api..."
	@mkdir -p "$(TARGET_DIR)"
	@for d in api/*/* ; do \
	  [ -d "$$d" ] || continue ; \
	  manifest="$$d/Cargo.toml" ; \
	  if [ -f "$$manifest" ]; then \
	    name=$$(awk -F= '/^\s*name\s*=\s*/ {gsub(/\"/,"",$$2); print $$2; exit}' "$$manifest" | tr -d " \t\r\n'\"") ; \
	    if [ -z "$$name" ]; then name=$$(basename "$$d" | tr "./" "__"); fi ; \
	    uname_s=$$(uname -s) ; \
	    case "$$uname_s" in \
	      *MINGW*|*MSYS*|*CYGWIN*) file="$$name.dll" ;; \
	      Darwin) file="lib$$name.dylib" ;; \
	      *) file="lib$$name.so" ;; \
	    esac ; \
	    dest="$(TARGET_DIR)/$${d#api/}" ; mkdir -p "$$dest" ; \
	    if [ -f "target/release/$$file" ]; then \
	      cp -f "target/release/$$file" "$$dest/" ; \
	      echo "  Copied $$file -> $$dest/" ; \
	    else \
	      echo "  ⚠️  Built artifact not found: target/release/$$file for plugin $$d" ; \
	    fi ; \
	  fi ; \
	done
	@echo "🔹 Plugins organized under $(TARGET_DIR)/api"

clean:
	@echo "🧹 Cleaning Cargo artifacts..."
	@cargo clean
	@rm -rf "$(TARGET_DIR)"
	@echo "✓ Clean complete"