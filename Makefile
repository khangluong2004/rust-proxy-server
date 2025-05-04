EXE=htproxy


# $(EXE): FORCE
# 	cargo build
# 	cp ./target/debug/htproxy ./htproxy

$(EXE): FORCE
	rustc ./src/main.rs --crate-name htproxy

FORCE: ;

clean:
	rm -rf ./target
	rm -rf ./htproxy

format:
	echo ""
