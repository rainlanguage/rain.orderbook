fn main() {
    cynic_codegen::register_schema("orderbook")
        .from_sdl_file("schema/orderbook.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}
