# audec

Small utility to detect compressed streams and automatically
decompress them.

## Example

```rust
use std::{io::BufReader, fs::File};

use audec::auto_decompress;

let input = File::open("maybe_compressed")?;
let mut input = auto_decompress(BufReader::new(input));
let mut decompressed = String::new();
input.read_to_string(&mut decompressed)?;
```

## Features

Each feature enables a decompression format

- `zlib-ng` (default)
- `zstd` (default)
- `bzip2`
- `lz4`
- `lz4_flex`
- `flate2`

`lz4` and `lz4_flex` are incompatible, at most one them can be
enabled. `zlib-ng` supersedes `flate2`.

License: GPL-3.0-or-later
