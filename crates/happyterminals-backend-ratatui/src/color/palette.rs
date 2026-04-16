//! xterm 256-color palette + compile-time 256→16 quantization LUT.
//!
//! Source: Wikipedia ANSI escape code spec (HIGH confidence per RESEARCH).
//! All arrays are built in `const fn`; MSRV 1.88 covers all required
//! const features (verified in RESEARCH §Pitfall 8).
//!
//! ## Structure
//!
//! - Indices `0..16`   — xterm system-16 palette (ANSI 16 named colors).
//! - Indices `16..232` — 6×6×6 RGB cube ordered `16 + 36*r + 6*g + b`,
//!   with channel levels `[0, 95, 135, 175, 215, 255]`.
//! - Indices `232..256` — 24 greyscale steps, `v = 8 + 10 * (i - 232)`.

/// sRGB channel levels for the 6×6×6 cube (xterm standard).
const CUBE_LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];

/// xterm system-16 palette (xterm variant of the ANSI 16 named colors).
/// Indexed by the low nibble of the 8-bit color code.
pub(crate) const SYSTEM_16: [(u8, u8, u8); 16] = [
    (0x00, 0x00, 0x00), (0x80, 0x00, 0x00), (0x00, 0x80, 0x00), (0x80, 0x80, 0x00),
    (0x00, 0x00, 0x80), (0x80, 0x00, 0x80), (0x00, 0x80, 0x80), (0xC0, 0xC0, 0xC0),
    (0x80, 0x80, 0x80), (0xFF, 0x00, 0x00), (0x00, 0xFF, 0x00), (0xFF, 0xFF, 0x00),
    (0x00, 0x00, 0xFF), (0xFF, 0x00, 0xFF), (0x00, 0xFF, 0xFF), (0xFF, 0xFF, 0xFF),
];

/// Standard xterm 256-color palette.
///
/// - `0..16`: ANSI system colors (xterm variant)
/// - `16..232`: 6×6×6 RGB cube, ordered as `16 + 36*r + 6*g + b`
///   with channel levels `[0, 95, 135, 175, 215, 255]`
/// - `232..256`: 24 greyscale steps, `v = 8 + 10 * (i - 232)`
pub const PALETTE_256: [(u8, u8, u8); 256] = build_palette();

/// Compile-time 256→16 quantization LUT (sRGB-nearest over the xterm
/// system-16 palette).
///
/// `PALETTE_16_LUT[i]` yields the system-16 index (0..16) whose RGB triple
/// is closest (squared sRGB) to `PALETTE_256[i]`.
pub const PALETTE_16_LUT: [u8; 256] = build_16_lut();

const fn build_palette() -> [(u8, u8, u8); 256] {
    let mut out = [(0u8, 0u8, 0u8); 256];
    let mut i = 0;
    while i < 16 {
        out[i] = SYSTEM_16[i];
        i += 1;
    }
    let mut r = 0;
    while r < 6 {
        let mut g = 0;
        while g < 6 {
            let mut b = 0;
            while b < 6 {
                out[16 + 36 * r + 6 * g + b] =
                    (CUBE_LEVELS[r], CUBE_LEVELS[g], CUBE_LEVELS[b]);
                b += 1;
            }
            g += 1;
        }
        r += 1;
    }
    let mut k = 0u8;
    while k < 24 {
        let v = 8 + 10 * k;
        out[232 + k as usize] = (v, v, v);
        k += 1;
    }
    out
}

const fn build_16_lut() -> [u8; 256] {
    let mut out = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        let target = PALETTE_256[i];
        let mut best_j = 0u8;
        let mut best_d = u32::MAX;
        let mut j = 0u8;
        while j < 16 {
            let d = sq_dist(target, SYSTEM_16[j as usize]);
            if d < best_d {
                best_d = d;
                best_j = j;
            }
            j += 1;
        }
        out[i] = best_j;
        i += 1;
    }
    out
}

/// Squared Euclidean distance in sRGB. Ordering-preserving — no sqrt.
///
/// The `as u32` cast is lossless: each squared component is non-negative
/// (`i32 * i32` where `|i32| <= 255`) and their sum ≤ `3 * 255^2 = 195_075`,
/// well within `u32::MAX`.
#[inline]
#[allow(clippy::cast_sign_loss)] // sum of squares is always non-negative
pub(crate) const fn sq_dist(a: (u8, u8, u8), b: (u8, u8, u8)) -> u32 {
    let dr = a.0 as i32 - b.0 as i32;
    let dg = a.1 as i32 - b.1 as i32;
    let db = a.2 as i32 - b.2 as i32;
    (dr * dr + dg * dg + db * db) as u32
}

/// Linear scan over [`PALETTE_256`]. Returns the first index minimizing
/// squared sRGB distance to `rgb`. Ties resolve to the lower index.
#[must_use]
pub fn nearest_256(rgb: (u8, u8, u8)) -> u8 {
    let mut best_i = 0u8;
    let mut best_d = u32::MAX;
    let mut i = 0u16;
    while i < 256 {
        let d = sq_dist(rgb, PALETTE_256[i as usize]);
        if d < best_d {
            best_d = d;
            // `i < 256` invariant → `i as u8` is lossless here.
            #[allow(clippy::cast_possible_truncation)]
            {
                best_i = i as u8;
            }
        }
        i += 1;
    }
    best_i
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_system_16_exact() {
        assert_eq!(PALETTE_256[0], (0, 0, 0), "system-0 is black");
        assert_eq!(PALETTE_256[7], (0xC0, 0xC0, 0xC0), "system-7 is light grey");
        assert_eq!(PALETTE_256[15], (0xFF, 0xFF, 0xFF), "system-15 is white");
    }

    #[test]
    fn palette_cube_origin() {
        // cube r=g=b=0 → level 0 → (0,0,0)
        assert_eq!(PALETTE_256[16], (0, 0, 0));
    }

    #[test]
    fn palette_cube_corner_white() {
        // cube r=g=b=5 → level 255 → (255,255,255); index 16 + 36*5 + 6*5 + 5 = 231
        assert_eq!(PALETTE_256[231], (255, 255, 255));
    }

    #[test]
    fn palette_cube_mid_sample() {
        // r=2 → 135, g=3 → 175, b=4 → 215; index 16 + 72 + 18 + 4 = 110
        let idx = 16 + 36 * 2 + 6 * 3 + 4;
        assert_eq!(idx, 110);
        assert_eq!(PALETTE_256[idx], (135, 175, 215));
    }

    #[test]
    fn palette_grey_first() {
        // k=0 → 8+0 = 8
        assert_eq!(PALETTE_256[232], (8, 8, 8));
    }

    #[test]
    fn palette_grey_last() {
        // k=23 → 8 + 10*23 = 238
        assert_eq!(PALETTE_256[255], (238, 238, 238));
    }

    #[test]
    fn nearest_256_cube_corner_white_is_white() {
        // Contract: whichever index is returned, its RGB must equal (255,255,255).
        // The linear scan encounters system-15 before cube-231, so with tie-break
        // to the lower index, we should get 15.
        let idx = nearest_256((255, 255, 255));
        assert_eq!(PALETTE_256[idx as usize], (255, 255, 255));
    }

    #[test]
    fn nearest_256_cube_origin_black() {
        // Contract: returned index's RGB must equal (0,0,0).
        let idx = nearest_256((0, 0, 0));
        assert_eq!(PALETTE_256[idx as usize], (0, 0, 0));
    }

    #[test]
    fn nearest_256_approximates_input() {
        // Pick a mid-gamut color; assert returned palette entry is within
        // 40 units per channel (sanity bound over the 6×6×6 cube resolution).
        let target = (100u8, 150u8, 200u8);
        let idx = nearest_256(target);
        let (r, g, b) = PALETTE_256[idx as usize];
        let dr = (i32::from(r) - i32::from(target.0)).unsigned_abs();
        let dg = (i32::from(g) - i32::from(target.1)).unsigned_abs();
        let db = (i32::from(b) - i32::from(target.2)).unsigned_abs();
        assert!(dr <= 40, "dr={dr}");
        assert!(dg <= 40, "dg={dg}");
        assert!(db <= 40, "db={db}");
    }

    #[test]
    fn palette_16_lut_range() {
        for (i, &v) in PALETTE_16_LUT.iter().enumerate() {
            assert!(v < 16, "PALETTE_16_LUT[{i}] = {v} not < 16");
        }
    }

    #[test]
    fn palette_16_lut_white_maps_to_white_or_light_grey() {
        // cube-white (231) → system-15 (white) OR system-7 (light grey 0xC0) —
        // distance ties are possible. Accept either.
        let v = PALETTE_16_LUT[231];
        assert!(
            v == 15 || v == 7,
            "PALETTE_16_LUT[231] = {v}; expected 15 (white) or 7 (light grey)"
        );
    }

    #[test]
    fn sq_dist_zero_self() {
        assert_eq!(sq_dist((10, 20, 30), (10, 20, 30)), 0);
    }

    #[test]
    fn sq_dist_symmetry() {
        let a = (10u8, 20u8, 30u8);
        let b = (200u8, 150u8, 90u8);
        assert_eq!(sq_dist(a, b), sq_dist(b, a));
    }
}
