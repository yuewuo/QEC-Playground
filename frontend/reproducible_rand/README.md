# Compile

Download https://github.com/krisselden/xoroshiro128starstar and run `npm install`

Replace `assembly/index.ts` with `index.ts` in this folder (to sync the behavior with Rust code)

run `npx asc --baseDir assembly -O3 -b index.wasm -t index.wat index.ts --sourceMap`

copy the compiled `assembly/index.wasm` and `assembly/wat` and `assembly/index.wasm.map` here
