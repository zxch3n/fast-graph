build-debug:
	cd crates/wasm && \
	RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals' \
		wasm-pack build -d ../../../wasm_dist --target web --debug . -- -Z build-std=panic_abort,std

build:
	cd crates/wasm && \
	RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals' \
		wasm-pack build -d ../../../wasm_dist --target web --release . -- -Z build-std=panic_abort,std
