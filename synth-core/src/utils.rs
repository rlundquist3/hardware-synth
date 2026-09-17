use micromath::F32Ext;

pub fn db_to_linear_gain(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

/**
 * The following constants and functrions were adapted from
 * https://github.com/daisy-embassy/daisy-embassy/blob/master/src/audio.rs
 * as these functions are not exported in the crate
 */

const SAMPLE_WIDTH_BITS: u32 = 32;

// Amplitude of a full-scale sample: `2^(SAMPLE_WIDTH_BITS - 1)`.
const SAMPLE_SCALE: f32 = (1u32 << (SAMPLE_WIDTH_BITS - 1)) as f32;

// Convert an audio sample from f32 (`-1.0..1.0`) to the wire format expected
// by the selected board's codec.
#[inline(always)]
pub fn f32_to_sample(x: f32) -> u32 {
    let x = x * SAMPLE_SCALE;
    let x = x.clamp(-SAMPLE_SCALE, SAMPLE_SCALE - 1.0);
    (x as i32) as u32
}

// Convert an audio sample from the selected board's codec wire format to
// f32 (`-1.0..1.0`).
#[inline(always)]
pub fn sample_to_f32(y: u32) -> f32 {
    // Sign-extend the SAMPLE_WIDTH_BITS-wide sample to i32 before normalizing.
    const SHIFT: u32 = 32 - SAMPLE_WIDTH_BITS;
    (((y << SHIFT) as i32) >> SHIFT) as f32 / SAMPLE_SCALE
}

#[cfg(test)]
mod tests {
    use crate::utils::db_to_linear_gain;
    use approx::assert_relative_eq;

    const GAIN_TOLERANCE: f32 = 0.01;

    #[test]
    fn no_gain() {
        assert_relative_eq!(db_to_linear_gain(0.0), 1.0);
    }
    #[test]
    fn positive_gain() {
        assert_relative_eq!(
            db_to_linear_gain(3.0),
            2.0_f32.sqrt(),
            max_relative = GAIN_TOLERANCE
        );
        assert_relative_eq!(db_to_linear_gain(6.0), 2.0, max_relative = GAIN_TOLERANCE);
        assert_relative_eq!(db_to_linear_gain(12.0), 4.0, max_relative = GAIN_TOLERANCE);
    }
    #[test]
    fn negative_gain() {
        assert_relative_eq!(
            db_to_linear_gain(-3.0),
            2.0_f32.sqrt() / 2.0,
            max_relative = GAIN_TOLERANCE
        );
        assert_relative_eq!(db_to_linear_gain(-6.0), 0.5, max_relative = GAIN_TOLERANCE);
        assert_relative_eq!(
            db_to_linear_gain(-12.0),
            0.25,
            max_relative = GAIN_TOLERANCE
        );
    }
}
