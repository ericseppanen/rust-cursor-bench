# Let's Talk About Rust Cursor Types

> This was originally published at [Code and Bitters].

I recently became curious about the different tools that are available to do one specific thing in Rust: provide an implementation of the `std::io::Write` trait for an in-memory buffer.

This might be useful if you have serializable data that you want to store in a database, or if you want to add checksums or perform compression before storing or sending some data. It may also be useful for buffering ordinary network writes (though [`BufWriter`] might be easier).

How can this be done in Rust? What's the most efficient way of buffering serialized data?

I use the name "cursor" for this kind of thing: an object that remembers the current write position in the buffer, so I don't have to do pointer math myself. That may not be the best name, but it's the one I'm going to use here.

#### Part 1: Cursor Choices

Here are the things I found that can be used as cursors:

There's a crate called [`bytes`] from the Tokio authors, that gets used in a lot of places. Among other things, it provides the [`BytesMut`] type that has the following properties:
- It has an internal refcount, so you can split one `BytesMut` into multiple non-overlapping `BytesMut` instances.
- It can support the [`Write`] trait, allowing the buffer to grow as bytes are written into it.

The standard library has a type [`Cursor`], which implements `Write`. It is has a generic parameter `Cursor<T>`, though the internal state of `Cursor` is private, so it's not possible to create a `Cursor` over your own buffer type. The sub-types that implement `Write` are:
- `Cursor<Vec<u8>>`
- `Cursor<&mut Vec<u8>>`
- `Cursor<&mut [u8]>`
- `Cursor<Box<[u8]>>`.

Those all look pretty similar— I'll discuss some of the differences in a moment.

The other option is really simple, and hides in plain sight: `&mut [u8]` can be used as a cursor type, and implements the `Write` trait. The current position is tracked by changing the reference itself, so if you want to write a function that doesn't consume the cursor, the code looks a little mysterious:
```rust
    fn write_some_data<W: Write>(w: &mut W) { todo!() }

    // My buffer
    let mut my_data = [0u8; 1024];
    // A cursor that implements Write.
    let mut my_cursor = &mut my_data[..];
    // Do some writing, but don't consume the cursor.
    write_some_data(&mut my_cursor);
```

All of the cursor types allow you to recover the internal buffer type afterwards using either an `.into_inner()` member function that consumes the cursor, or by the caller keeping ownership of the original buffer.

#### Part 2: Cursor Flavors

Which of these should we choose? There are three qualities we might care about:
- Refcounted
- Growable
- Seekable / Overwritable

The `BytesMut` trait is the only internally-refcounted option. This type is used in Tokio, so it may be the right choice if you want to use e.g. the Tokio [`AsyncReadExt`] trait.

`BytesMut` also has the other two qualities: growable (the buffer will expand when you write more data into it), and seekable+overwritable (you can seek back to an arbitrary point and write more data).

Of the `Cursor` flavors, only the `Vec` ones are growable. In fact, `Cursor<&mut Vec<u8>>` and `Cursor<Vec<u8>>` have identical behavior in every way, so they're pretty much equivalent in practice.

The other options, `Cursor<&mut [u8]>`, `Cursor<Box<[u8]>>`, and `&mut [u8]` all wrap a `[u8]` slice— these types are not growable.

The last one, `&mut [u8]`, is the only option that is not growable or seekable (unless you manually change the reference between write operations).

The `Write` trait isn't very complicated; we can even create our own cursor type. Here's one that is growable but not seekable:
```rust
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
        let len = buf.len();
        self.0.extend_from_slice(buf);
        Ok(len)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

```

#### Part 3: Cursor Benchmarks

Buffering data is something that we'd like to be as efficient as possible; in many cases we might even hope that the compiler can "see through" abstractions and optimize away the buffering entirely.

Because `BytesMut` has internal "magic": (refcounts and pointers and some unsafe logic), we might be concerned that this adds performance overhead. But that actually turns out not to be the case: `BytesMut` is significantly faster than `Cursor`.

In general, the performance seems to be:
- Slowest: Cursors using `Vec`
- Middle: Cursors that are growable
- Fastest: Cursors using slices

I created a quick benchmark using [Criterion] to compare the various cursor types. Each test run creates a 32KB buffer, and uses `serde` with the `bincode` format to serialize a simple 16-byte data structure 2048 times.

Here is the raw data:

| cursor type            | time to serialize 32KB |
|------------------------|------------------------|
| `Cursor<Vec<u8>>`      | 102 us                 |
| `Cursor<&mut Vec<u8>>` | 98 us                  |
| `WriterVec`            | 66 us                  |
| `BytesMut`             | 45 us                  |
| `Cursor<&mut [u8]>`    | 37 us                  |
| `Cursor<Box<[u8]>>`    | 37 us                  |
| `&mut [u8]`            | 28 us                  |

This data was gathered on my laptop with no attempt to stabilize the CPU clock speed, so take it with a grain of salt: the numbers move around ~5% from one run to the next. I also made no attempt to remove allocator overhead from the benchmark. If you'd like to experiment with the benchmark yourself, the entire project is on GitHub [here](https://github.com/ericseppanen/rust-cursor-bench).

Please raise a GitHub issue if you find something wrong with my benchmark methodology. Micro-benchmarking can often show misleading results, so I'm very interested to learn if there's something I've done wrong.

[`BufWriter`]: https://doc.rust-lang.org/std/io/struct.BufWriter.html
[`Write`]: https://doc.rust-lang.org/std/io/trait.Write.html
[`Cursor`]: https://doc.rust-lang.org/std/io/struct.Cursor.html
[`bytes`]: https://docs.rs/bytes/latest/bytes/index.html
[`BytesMut`]: https://docs.rs/bytes/latest/bytes/struct.BytesMut.html
[`AsyncReadExt`]: https://docs.rs/tokio/1.6.1/tokio/io/trait.AsyncReadExt.html
[Criterion]: https://github.com/bheisler/criterion.rs
[Code and Bitters]: https://codeandbitters.com/rust-cursors/
