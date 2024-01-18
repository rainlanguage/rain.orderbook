fn main() {
    cynic_codegen::register_schema("orders")
        .from_sdl_file("schemas/orders.schema.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}
