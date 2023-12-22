//! Small utility to detect compressed streams and automatically
//! decompress them.
//!
//! # Example
//!
//! ```rust,no_run
//!# fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use std::{io::BufReader, fs::File};
//!
//! use audec::auto_decompress;
//!
//! let input = File::open("maybe_compressed")?;
//! let mut input = auto_decompress(BufReader::new(input));
//! let mut decompressed = String::new();
//! input.read_to_string(&mut decompressed)?;
//!# Ok(())
//!# }
//! ```
//!
//! # Features
//!
//! Each feature enables a decompression format
//!
//! - `zlib-ng` (default)
//! - `zstd` (default)
//! - `bzip2`
//! - `lz4`
//! - `lz4_flex`
//! - `flate2`
//!
//! `lz4` and `lz4_flex` are incompatible, at most one them can be
//! enabled. `zlib-ng` supersedes `flate2`.
use std::io::{BufRead, BufReader};

use log::debug;

#[cfg(all(feature = "lz4", feature = "lz4_flex"))]
compile_error!("feature \"lz4\" and feature \"lz4_flex\" cannot be enabled at the same time");

/// Automatic decompression
///
/// Returns a new stream that automatically decompresses the output of
/// the original source. The format is determined by looking at the
/// leading magic bytes. Returns the original source if the magic
/// bytes are not recognized or not enough bytes can be read to
/// determine the format.
pub fn auto_decompress<'a, B: 'a + BufRead>(mut r: B) -> Box<dyn BufRead + 'a> {
    let bytes = match r.fill_buf() {
        Ok(bytes) => bytes,
        Err(_) => return Box::new(r),
    };
    match bytes {
        #[cfg(feature = "bzip2")]
        [b'B', b'Z', b'h', ..] => {
            debug!("Decompress as bzip2");
            Box::new(BufReader::new(bzip2::read::BzDecoder::new(r)))
        },
        #[cfg(feature = "flate2")]
        &[0x1f, 0x8b, ..] => {
            debug!("Decompress as gzip");
            Box::new(BufReader::new(flate2::read::GzDecoder::new(r)))
        },
        #[cfg(feature = "lz4")]
        &[0x04, 0x22, 0x4d, 0x18, ..] => {
            debug!("Decompress as lz4");
            Box::new(BufReader::new(lz4::Decoder::new(r).unwrap()))
        },
        #[cfg(feature = "lz4_flex")]
        &[0x04, 0x22, 0x4d, 0x18, ..] => {
            debug!("Decompress as lz4");
            Box::new(BufReader::new(lz4_flex::frame::FrameDecoder::new(r)))
        },
        #[cfg(feature = "zstd")]
        &[0x28, 0xb5, 0x2f, 0xfd, ..] => {
            debug!("Decompress as zstd");
            Box::new(BufReader::new(zstd::stream::Decoder::new(r).unwrap()))
        },
        _ => {
            debug!("No decompression");
            Box::new(r)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "bzip2")]
    #[test]
    fn bzip2_empty() {
        let source = [
            0x42, 0x5a, 0x68, 0x39, 0x17, 0x72, 0x45, 0x38, 0x50, 0x90, 0x00,
            0x00, 0x00, 0x00
        ].as_slice();
        let mut reader = auto_decompress(source);
        let mut buf = Vec::new();
        assert_eq!(reader.read_to_end(&mut buf).unwrap(), 0)
    }

    #[cfg(feature = "flate2")]
    #[test]
    fn flate2_empty() {
        let source = [
            0x1f, 0x8b, 0x08, 0x08, 0x7e, 0x70, 0xca, 0x64, 0x00, 0x03, 0x66,
            0x6f, 0x6f, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00
        ].as_slice();
        let mut reader = auto_decompress(source);
        let mut buf = Vec::new();
        assert_eq!(reader.read_to_end(&mut buf).unwrap(), 0)
    }

    #[cfg(any(feature = "lz4", feature = "lz4_flex"))]
    #[test]
    fn lz4_empty() {
        let source = [
            0x04, 0x22, 0x4d, 0x18, 0x64, 0x40, 0xa7, 0x00, 0x00, 0x00, 0x00,
            0x05, 0x5d, 0xcc, 0x02
        ].as_slice();
        let mut reader = auto_decompress(source);
        let mut buf = Vec::new();
        assert_eq!(reader.read_to_end(&mut buf).unwrap(), 0)
    }

    #[cfg(feature = "zstd")]
    #[test]
    fn zstd_empty() {
        let source = [
            0x28, 0xb5, 0x2f, 0xfd, 0x24, 0x00, 0x01, 0x00, 0x00, 0x99, 0xe9,
            0xd8, 0x51
        ].as_slice();
        let mut reader = auto_decompress(source);
        let mut buf = Vec::new();
        assert_eq!(reader.read_to_end(&mut buf).unwrap(), 0)
    }

    #[test]
    fn empty() {
        let source = [].as_slice();
        let mut reader = auto_decompress(source);
        let mut buf = Vec::new();
        assert_eq!(reader.read_to_end(&mut buf).unwrap(), 0)
    }

}
