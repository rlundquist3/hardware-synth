use micromath::F32Ext;

pub fn db_to_linear_gain(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}
