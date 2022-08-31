# bitmac
This library provides implementation of bitmap with custom bit accessing and resizing strategy.

[<img alt="crates.io" src="https://img.shields.io/crates/v/bitmac?style=flat-square">](https://crates.io/crates/bitmac)
[<img alt="docs.rs" src="https://img.shields.io/docsrs/bitmac?style=flat-square">](https://docs.rs/bitmac)
[<img alt="build" src="https://img.shields.io/github/workflow/status/LazyMechanic/bitmac/Rust?style=flat-square">](https://github.com/LazyMechanic/bitmac/actions)

```toml
[dependencies]
bitmac = "0.3"
```

### Features

| Feature    | Description                                                                                                                   |
|------------|-------------------------------------------------------------------------------------------------------------------------------|
| `bytes`    | to implement `ContainerRead` trait for `Bytes` and `ContainerRead`, `ContainerWrite`, and `Resizable` traits for [`BytesMut`] |
| `smallvec` | to implement `ContainerRead`, `ContainerWrite` and `Resizable` traits for `SmallVec`                                          |

### Example
```rust
use bitmac::{StaticBitmap, LSB, Intersection, Union};

fn main() {
    let mut bitmap = StaticBitmap::<u16, LSB>::default();

    assert!(!bitmap.get(0));
    assert!(!bitmap.get(7));
    
    bitmap.set(0, true);
    bitmap.set(7, true);
    assert!(bitmap.get(0));
    assert!(bitmap.get(7));
    
    assert_eq!(bitmap.intersection_len(0b0000_1111_0000_0001u16), 1);
    assert_eq!(bitmap.union_len(0b0000_1111_0000_0001u16), 6);
}
```

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>