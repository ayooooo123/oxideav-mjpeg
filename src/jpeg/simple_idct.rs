// SPDX-License-Identifier: LGPL-2.1-or-later
// Port of FFmpeg 2da55bf libavcodec/simple_idct_template.c at BIT_DEPTH 8
// with int16 input (`ff_simple_idct_put_int16_8bit`): the C IDCT that
// `-idct simple` selects, with the 64-bit DC-only row shortcut FFmpeg builds
// on arm64 and x86-64 (HAVE_FAST_64BIT).
// Copyright (c) 2001 Michael Niedermayer <michaelni@gmx.at>
//
// This file is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License as published by the
// Free Software Foundation; either version 2.1 of the License, or (at your
// option) any later version. See LICENSE-LGPL.

//! FFmpeg's simple IDCT for 8-bit samples. The C code's unsigned
//! intermediates (`SUINT`) are wrapping `i32` arithmetic here; each row is
//! stored back as `int16_t`, and the columns are clipped to `0..=255`.

const W1: i32 = 22725;
const W2: i32 = 21407;
const W3: i32 = 19266;
const W4: i32 = 16383;
const W5: i32 = 12873;
const W6: i32 = 8867;
const W7: i32 = 4520;
const ROW_SHIFT: u32 = 11;
const COL_SHIFT: u32 = 20;
const DC_SHIFT: u32 = 3;

/// `idctRowCondDC` on one row of eight coefficients.
fn row(r: &mut [i16]) {
    if r[1..8].iter().all(|&v| v == 0) {
        // The 64-bit DC-only shortcut: (row[0] << 3) & 0xffff, eight times.
        let v = (i32::from(r[0]) << DC_SHIFT) as i16;
        r[..8].fill(v);
        return;
    }
    let x = |i: usize| i32::from(r[i]);
    let mut a0 = W4.wrapping_mul(x(0)).wrapping_add(1 << (ROW_SHIFT - 1));
    let (mut a1, mut a2, mut a3) = (a0, a0, a0);
    a0 = a0.wrapping_add(W2.wrapping_mul(x(2)));
    a1 = a1.wrapping_add(W6.wrapping_mul(x(2)));
    a2 = a2.wrapping_sub(W6.wrapping_mul(x(2)));
    a3 = a3.wrapping_sub(W2.wrapping_mul(x(2)));

    let mut b0 = W1.wrapping_mul(x(1)).wrapping_add(W3.wrapping_mul(x(3)));
    let mut b1 = W3.wrapping_mul(x(1)).wrapping_add((-W7).wrapping_mul(x(3)));
    let mut b2 = W5.wrapping_mul(x(1)).wrapping_add((-W1).wrapping_mul(x(3)));
    let mut b3 = W7.wrapping_mul(x(1)).wrapping_add((-W5).wrapping_mul(x(3)));

    if r[4..8].iter().any(|&v| v != 0) {
        a0 = a0.wrapping_add(W4.wrapping_mul(x(4)).wrapping_add(W6.wrapping_mul(x(6))));
        a1 = a1.wrapping_add((-W4).wrapping_mul(x(4)).wrapping_sub(W2.wrapping_mul(x(6))));
        a2 = a2.wrapping_add((-W4).wrapping_mul(x(4)).wrapping_add(W2.wrapping_mul(x(6))));
        a3 = a3.wrapping_add(W4.wrapping_mul(x(4)).wrapping_sub(W6.wrapping_mul(x(6))));

        b0 = b0
            .wrapping_add(W5.wrapping_mul(x(5)))
            .wrapping_add(W7.wrapping_mul(x(7)));
        b1 = b1
            .wrapping_add((-W1).wrapping_mul(x(5)))
            .wrapping_add((-W5).wrapping_mul(x(7)));
        b2 = b2
            .wrapping_add(W7.wrapping_mul(x(5)))
            .wrapping_add(W3.wrapping_mul(x(7)));
        b3 = b3
            .wrapping_add(W3.wrapping_mul(x(5)))
            .wrapping_add((-W1).wrapping_mul(x(7)));
    }

    r[0] = (a0.wrapping_add(b0) >> ROW_SHIFT) as i16;
    r[7] = (a0.wrapping_sub(b0) >> ROW_SHIFT) as i16;
    r[1] = (a1.wrapping_add(b1) >> ROW_SHIFT) as i16;
    r[6] = (a1.wrapping_sub(b1) >> ROW_SHIFT) as i16;
    r[2] = (a2.wrapping_add(b2) >> ROW_SHIFT) as i16;
    r[5] = (a2.wrapping_sub(b2) >> ROW_SHIFT) as i16;
    r[3] = (a3.wrapping_add(b3) >> ROW_SHIFT) as i16;
    r[4] = (a3.wrapping_sub(b3) >> ROW_SHIFT) as i16;
}

/// `idctSparseColPut` on column `c`, written to `dest` down from
/// `dest[c]`, one row every `stride` bytes.
fn col_put(block: &[i16; 64], c: usize, dest: &mut [u8], stride: usize) {
    let x = |row: usize| i32::from(block[8 * row + c]);
    let mut a0 = W4.wrapping_mul(x(0) + (1 << (COL_SHIFT - 1)) / W4);
    let (mut a1, mut a2, mut a3) = (a0, a0, a0);
    a0 = a0.wrapping_add(W2.wrapping_mul(x(2)));
    a1 = a1.wrapping_add(W6.wrapping_mul(x(2)));
    a2 = a2.wrapping_add((-W6).wrapping_mul(x(2)));
    a3 = a3.wrapping_add((-W2).wrapping_mul(x(2)));

    let mut b0 = W1.wrapping_mul(x(1));
    let mut b1 = W3.wrapping_mul(x(1));
    let mut b2 = W5.wrapping_mul(x(1));
    let mut b3 = W7.wrapping_mul(x(1));
    b0 = b0.wrapping_add(W3.wrapping_mul(x(3)));
    b1 = b1.wrapping_add((-W7).wrapping_mul(x(3)));
    b2 = b2.wrapping_add((-W1).wrapping_mul(x(3)));
    b3 = b3.wrapping_add((-W5).wrapping_mul(x(3)));

    if x(4) != 0 {
        a0 = a0.wrapping_add(W4.wrapping_mul(x(4)));
        a1 = a1.wrapping_add((-W4).wrapping_mul(x(4)));
        a2 = a2.wrapping_add((-W4).wrapping_mul(x(4)));
        a3 = a3.wrapping_add(W4.wrapping_mul(x(4)));
    }
    if x(5) != 0 {
        b0 = b0.wrapping_add(W5.wrapping_mul(x(5)));
        b1 = b1.wrapping_add((-W1).wrapping_mul(x(5)));
        b2 = b2.wrapping_add(W7.wrapping_mul(x(5)));
        b3 = b3.wrapping_add(W3.wrapping_mul(x(5)));
    }
    if x(6) != 0 {
        a0 = a0.wrapping_add(W6.wrapping_mul(x(6)));
        a1 = a1.wrapping_add((-W2).wrapping_mul(x(6)));
        a2 = a2.wrapping_add(W2.wrapping_mul(x(6)));
        a3 = a3.wrapping_add((-W6).wrapping_mul(x(6)));
    }
    if x(7) != 0 {
        b0 = b0.wrapping_add(W7.wrapping_mul(x(7)));
        b1 = b1.wrapping_add((-W5).wrapping_mul(x(7)));
        b2 = b2.wrapping_add(W3.wrapping_mul(x(7)));
        b3 = b3.wrapping_add((-W1).wrapping_mul(x(7)));
    }

    let px = |v: i32| (v >> COL_SHIFT).clamp(0, 255) as u8;
    let out = [
        a0.wrapping_add(b0),
        a1.wrapping_add(b1),
        a2.wrapping_add(b2),
        a3.wrapping_add(b3),
        a3.wrapping_sub(b3),
        a2.wrapping_sub(b2),
        a1.wrapping_sub(b1),
        a0.wrapping_sub(b0),
    ];
    for (j, v) in out.into_iter().enumerate() {
        dest[j * stride + c] = px(v);
    }
}

/// `ff_simple_idct_put_int16_8bit`: inverse-transforms the natural-order
/// `block` in place and writes the 8×8 clipped pixels to `dest` (row `j`
/// at `dest[j * stride..]`).
pub fn simple_idct_put(block: &mut [i16; 64], dest: &mut [u8], stride: usize) {
    for r in block.chunks_exact_mut(8) {
        row(r);
    }
    for c in 0..8 {
        col_put(block, c, dest, stride);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A DC-only block: FFmpeg's DC 1024 (level 0 plus the +1024 its JPEG
    /// predictor starts from) is mid-grey everywhere.
    #[test]
    fn dc_only_block_is_flat() {
        let mut block = [0i16; 64];
        block[0] = 1024;
        let mut out = [0u8; 64];
        simple_idct_put(&mut block, &mut out, 8);
        assert!(out.iter().all(|&p| p == 128), "{out:?}");
    }
}
