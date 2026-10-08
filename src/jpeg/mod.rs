//! Baseline JPEG bitstream primitives.

#[doc(hidden)]
pub mod arith;
#[doc(hidden)]
pub mod color;
#[doc(hidden)]
pub mod dct;
#[doc(hidden)]
pub mod huffman;
pub mod inspect;
#[doc(hidden)]
pub mod markers;
#[doc(hidden)]
pub mod parser;
#[doc(hidden)]
pub mod quant;
/// FFmpeg's C simple IDCT (LGPL-2.1-or-later, see the file's notice).
pub mod simple_idct;
#[doc(hidden)]
pub mod zigzag;
