use bincode::Options;
use bytes::{BufMut, BytesMut};
use serde::Serialize;
use std::io::{Write, Cursor};

pub fn be_coder() -> impl Options {
    bincode::DefaultOptions::new()
        .with_big_endian()
        .with_fixint_encoding()
        .allow_trailing_bytes()
}

#[derive(Debug, Default, Serialize)]
pub struct MyStruct {
    a: u64,
    b: u32,
    c: u8,
    d: bool,
    e: u16,
}

impl MyStruct {
    pub fn new(ii: usize) -> Self {
        MyStruct {
            a: ii as u64 + 1,
            b: ii as u32 + 2,
            c: ii as u8 + 3,
            d: ii & 1 != 0,
            e: 0xAA55,
        }
    }
}

#[inline(never)]
pub fn serialize_it<T, W>(item: &T, sink: &mut W)
where
    T: serde::Serialize,
    W: Write,
{
    // Throw away errors; not very realistic!
    be_coder().serialize_into(sink, &item).unwrap();
}

#[inline(never)]
pub fn test_serialize_cursor(s: MyStruct) {
    let mut c = Cursor::new(Vec::<u8>::new());
    serialize_it(&s, &mut c);
    //println!("{:?}", c.into_inner());
}

#[inline(never)]
pub fn test_serialize_bytesmut(s: MyStruct) {
    let mut b = BytesMut::new().writer();
    serialize_it(&s, &mut b);
    //println!("{:?}", b.into_inner());
}



pub struct WriterVec(Vec<u8>);

impl WriterVec {
    pub fn new() -> Self {
        WriterVec(Vec::new())
    }

    pub fn with_capacity(n: usize) -> Self {
        WriterVec(Vec::with_capacity(n))
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

impl Write for WriterVec {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
