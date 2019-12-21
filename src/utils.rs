pub fn apm_to_hex(value: u64) -> String {
    // this will need tweaking over time
    let constrained_hexed = (value as f64) / 4.0 / 16.0;
    let (first, second) = (constrained_hexed.floor(), constrained_hexed.fract() * 16.0);
    return format!("{:x}{:x}0000", first as i8, second as i8);
}
