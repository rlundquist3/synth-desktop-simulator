pub const A440: f32 = 440.0;

pub fn get_freq_for_note(semitones_from_a440: i32) -> f32 {
    let exponent = semitones_from_a440 as f32 / 12.0;
    A440 * 2.0_f32.powf(exponent)
}
