//! Helpers to serialize and deserialize Roaring bitmaps.
//!
//! These utilities convert a `roaring::RoaringBitmap` to/from a compact byte
//! representation suitable for storage in the database (e.g., the
//! `asset_coverage_bitmap.bitmap` BLOB column).
//!
//! Round-trip usage:
//! ```no_run
//! use asset_sync::roaring_bytes::{rb_from_bytes, rb_to_bytes};
//! use roaring::RoaringBitmap;
//!
//! let mut rb = RoaringBitmap::new();
//! rb.insert(1);
//! rb.insert(10);
//!
//! let bytes = rb_to_bytes(&rb);
//! let rb2 = rb_from_bytes(&bytes);
//! assert_eq!(rb, rb2);
//! ```

use std::io::Cursor;

use roaring::RoaringBitmap;

/// Serialize a Roaring bitmap into a compact byte vector.
///
/// The format is the standard portable serialization used by `roaring`.
///
/// Panics:
/// - If serialization fails (I/O into the in-memory buffer).
pub fn rb_to_bytes(rb: &RoaringBitmap) -> Vec<u8> {
    let mut buf = Vec::with_capacity(rb.serialized_size());
    rb.serialize_into(&mut buf).expect("serialize roaring");
    buf
}

/// Deserialize a Roaring bitmap from bytes previously produced by [`rb_to_bytes`].
///
/// Panics:
/// - If the provided bytes are not a valid serialized Roaring bitmap.
pub fn rb_from_bytes(bytes: &[u8]) -> RoaringBitmap {
    RoaringBitmap::deserialize_from(Cursor::new(bytes)).expect("deserialize roaring")
}

#[cfg(test)]
mod tests {
    use super::*;
    use roaring::RoaringBitmap;

    #[test]
    fn round_trip_small_bitmap() {
        let mut rb = RoaringBitmap::new();
        rb.insert(0);
        rb.insert(1);
        rb.insert(2);
        rb.insert(10);
        rb.insert(65_535);

        let bytes = rb_to_bytes(&rb);
        assert!(!bytes.is_empty(), "serialized bytes should not be empty");

        let rb2 = rb_from_bytes(&bytes);
        assert_eq!(rb, rb2, "bitmap should round-trip via bytes");
    }

    #[test]
    fn round_trip_empty_bitmap() {
        let rb = RoaringBitmap::new();
        let bytes = rb_to_bytes(&rb);
        let rb2 = rb_from_bytes(&bytes);
        assert_eq!(rb, rb2, "empty bitmap should round-trip");
    }
}
