# bitmac
This library provides implementation of bitmap with custom bit accessing and resizing strategy.

[<img alt="crates.io" src="https://img.shields.io/crates/v/bitmac?style=flat-square">](https://crates.io/crates/bitmac)
[<img alt="docs.rs" src="https://img.shields.io/docsrs/bitmac?style=flat-square">](https://docs.rs/bitmac)
[<img alt="build" src="https://img.shields.io/github/workflow/status/LazyMechanic/bitmac/Rust?style=flat-square">](https://github.com/LazyMechanic/bitmac/actions)

```toml
[dependencies]
bitmac = "0.2"
```

### Features
- `bytes` - implemented trait `ContainerMut` for `BytesMut`
- `smallvec` - implemented trait `ContainerMut` for `SmallVec`

### Example
```rust
use bitmac::{Bitmap, MinimumRequiredStrategy, LSB};

fn main() {
    let mut bitmap = Bitmap::<Vec<u8>, MinimumRequiredStrategy, LSB>::default();

    assert_eq!(bitmap.as_bytes().len(), 0);
    
    bitmap.set(0, true);
    bitmap.set(7, true);
    assert_eq!(bitmap.as_bytes().len(), 1);
    
    bitmap.set(15, true);
    assert_eq!(bitmap.as_bytes().len(), 2);

    assert!(bitmap.get(0));
    assert!(bitmap.get(7));
    assert!(bitmap.get(15));

    assert!(!bitmap.get(1));
    assert!(!bitmap.get(8));
    assert!(!bitmap.get(300));

    assert_eq!(bitmap.as_bytes().len(), 2);
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