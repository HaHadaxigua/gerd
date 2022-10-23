mod wasmtime;

use wasmer::{imports, wat2wasm, Instance, instantiate, Module, Store, Value, Val};

fn main() -> anyhow::Result<()> {
	let mut store = Store::default();
	let module = Module::from_file(&store, "build/main.wasm")?;
	let import_object = imports! {};
	let instance = Instance::new(&module, &import_object)?;

	let main = instance.exports.get_function("multiply")?;
	let result = main.call(&vec![Value::I32(1), Value::I32(2)]).unwrap();
	println!("{:?}", result);

	Ok(())
}