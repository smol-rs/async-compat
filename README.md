# async-compat

[![Build](https://github.com/stjepang/async-compat/workflows/Build%20and%20test/badge.svg)](
https://github.com/stjepang/async-compat/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](
https://github.com/stjepang/async-compat)
[![Cargo](https://img.shields.io/crates/v/async-compat.svg)](
https://crates.io/crates/async-compat)
[![Documentation](https://docs.rs/async-compat/badge.svg)](
https://docs.rs/async-compat)

Compatibility adapter between tokio and futures.

There are two kinds of compatibility issues between [tokio] and [futures]:

- Tokio's types cannot be used outside the tokio context. Any attempt to use
them will panic. When the `Compat` adapter is applied to a future, it will enter the
tokio context.

- Tokio and futures have similar but different I/O traits `AsyncRead`, `AsyncWrite`,
`AsyncBufRead`, and `AsyncSeek`. When the `Compat` adapter is applied to an I/O type, it
will implement traits of the opposite kind.

You can apply the `Compat` adapter using the `Compat::new()` constructor or using any
method from the `CompatExt` trait.

## Examples

This program reads lines from stdin and echoes them into stdout:

```rust
fn main() -> std::io::Result<()> {
    futures::executor::block_on(async {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();

        // The following line fails for two reasons:
        // - Compilation error due mismatched `AsyncRead` and `AsyncWrite` traits.
        // - Runtime error because stdin and stdout are used outside tokio context.
        futures::io::copy(stdin, &mut stdout).await?;
        Ok(())
    })
}
```

To get around the compatibility issues, apply the `Compat` adapter:

```rust
use async_compat::CompatExt;

fn main() -> std::io::Result<()> {
    futures::executor::block_on(async {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();

        futures::io::copy(stdin.compat(), &mut stdout.compat_mut()).compat().await?;
        Ok(())
    })
}
```

It is also possible to apply `Compat` to the outer future passed to
[`futures::executor::block_on()`] rather than [`futures::io::copy()`] itself:

```rust
use async_compat::{Compat, CompatExt};

fn main() -> std::io::Result<()> {
    futures::executor::block_on(Compat::new(async {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();

        futures::io::copy(stdin.compat(), &mut stdout.compat_mut()).await?;
        Ok(())
    }))
}
```

The compatibility adapter converts between tokio-based and futures-based I/O types in any
direction. Here's how we can write the same program by using futures-based I/O types inside
tokio:

```rust
use async_compat::CompatExt;
use blocking::Unblock;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut stdin = Unblock::new(std::io::stdin());
    let mut stdout = Unblock::new(std::io::stdout());

    tokio::io::copy(&mut stdin.compat_mut(), &mut stdout.compat_mut()).await?;
    Ok(())
}
```

Finally, we can use any tokio-based crate from any other async runtime.
Here are [reqwest] and [warp] as an example:

```rust
use async_compat::{Compat, CompatExt};
use warp::Filter;

fn main() {
    futures::executor::block_on(Compat::new(async {
        // Make an HTTP GET request.
        let response = reqwest::get("https://www.rust-lang.org").await.unwrap();
        println!("{}", response.text().await.unwrap());

        // Start an HTTP server.
        let routes = warp::any().map(|| "Hello from warp!");
        warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
    }))
}
```

[blocking]: https://docs.rs/blocking
[futures]: https://docs.rs/futures
[reqwest]: https://docs.rs/reqwest
[tokio]: https://docs.rs/tokio
[warp]: https://docs.rs/warp
[`futures::io::copy()`]: https://docs.rs/futures/0.3/futures/io/fn.copy.html
[`futures::executor::block_on()`]: https://docs.rs/futures/0.3/futures/executor/fn.block_on.html

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

#### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
