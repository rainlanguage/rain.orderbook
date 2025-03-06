#[macro_use]
extern crate rain_orderbook_macros;

struct TestStruct;

#[impl_wasm_exports]
impl TestStruct {
    #[wasm_export(unchecked_return_type = string)]
    pub async fn some_static_method(arg: String) -> Result<String, Error> {
        Ok(String::new())
    }
}

fn main() {}
