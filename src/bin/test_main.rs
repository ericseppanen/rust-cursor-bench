use rust_cursor_bench::{MyStruct, test_serialize_bytesmut, test_serialize_cursor};

fn main() {
    test_serialize_cursor(MyStruct::default());
    test_serialize_bytesmut(MyStruct::default());
}
