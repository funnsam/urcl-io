run: build
	python3 -m http.server

build:
	cargo clean -p urcl-io --release
	wasm-pack build --target web
	mv ./pkg/urcl_io_bg.wasm ./pkg/urcl_io_tmp.wasm
	wasm-opt -O3 -o ./pkg/urcl_io_bg.wasm ./pkg/urcl_io_tmp.wasm
	rm ./pkg/.gitignore
	rm ./pkg/urcl_io_tmp.wasm
