

all:
	RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build -j 8 -p oc_worker --target wasm32-unknown-unknown --release
	wasm-bindgen target/wasm32-unknown-unknown/release/oc_worker.wasm --out-dir pkg --target web
clean:
	cargo clean
	rm -rf ./pkg
