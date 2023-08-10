use nbtrock::{wasm, IResult, NBT};
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test() -> IResult<()> {
    let f = [
        10, 0, 0, 3, 4, 0, 101, 100, 105, 116, 130, 251, 11, 0, 10, 4, 0, 78, 101, 120, 116, 2, 5,
        0, 115, 104, 111, 114, 116, 10, 0, 7, 9, 0, 66, 121, 116, 101, 65, 114, 114, 97, 121, 3, 0,
        0, 0, 8, 9, 7, 0, 0,
    ];
    let n = NBT::from_reader(&mut f.as_slice())?;
    let nbt = wasm::NBT(n);
    let _ = wasm::NBT::new(nbt.value().unwrap()).unwrap();
    Ok(())
}
