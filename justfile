wasm_dir := "sm213_parser_wasm"
frontend_dir := "sm213_editor"
default:
    @just --list
buildcopy: build copy
build:
    cd {{wasm_dir}} && wasm-pack build --target web
buildrelease:
    cd {{wasm_dir}} && wasm-pack build --target web
    cd {{frontend_dir}} && npm run build
    cp {{frontend_dir}}/dist . -r
copy:
    -mkdir {{frontend_dir}}/src/wasm
    rm {{wasm_dir}}/pkg/package.json
    cp {{wasm_dir}}/pkg/* {{frontend_dir}}/src/wasm/
watchwasm:
    watchexec -w {{wasm_dir}}/src just buildcopy
watchvite:
    cd {{frontend_dir}} && npm run dev
