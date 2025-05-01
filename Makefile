EXE=htproxy


$(EXE): FORCE
	cargo build
	cp ./target/debug/htproxy ./htproxy

FORCE: ;

clean:
	rm -rf ./target
	rm -rf ./htproxy

format:
	echo ""
