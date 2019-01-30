
.PHONY: apng benchmark

apngc:
	(cd example ; cargo build --release)

benchmark:
	cargo +nightly bench --features benchmark
