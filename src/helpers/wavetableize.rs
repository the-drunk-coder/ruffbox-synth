use crate::building_blocks::interpolation::interpolate;

/// first, naive implementation to chop a sample buffer to a wavetable
pub fn wavetableize_raw(
    buffer: &[f32],
    mut matrix_size: (usize, usize),
    start: f32,
) -> Vec<Vec<f32>> {
    // check if final size is possible,
    // correct if necessary
    if buffer.len() / matrix_size.1 < matrix_size.0 {
        matrix_size.0 = buffer.len() / matrix_size.1;
    }

    let mut wavematrix = Vec::new();

    let start: usize = (buffer.len() as f32 * start) as usize;

    for i in 0..(matrix_size.0 - 1) {
        wavematrix.push(
            buffer[(start + i * matrix_size.1)..(start + ((i + 1) * matrix_size.1))].to_vec(),
        );
    }

    wavematrix
}

pub fn wavetableize_zerocrossing(
    buffer: &[f32],
    mut matrix_size: (usize, usize),
    start: f32,
) -> Vec<Vec<f32>> {
    // check if final size is possible,
    // correct if necessary
    if buffer.len() / matrix_size.1 < matrix_size.0 {
        matrix_size.0 = buffer.len() / matrix_size.1;
    }

    let mut wavematrix = Vec::new();

    let start: usize = (buffer.len() as f32 * start) as usize;

    for i in 0..(matrix_size.0 - 1) {
        let mut raw_buffer =
            buffer[(start + i * matrix_size.1)..(start + ((i + 1) * matrix_size.1))].to_vec();

        println!("before {:?}", raw_buffer);

        // find zerocrossing from new to pos
        let mut start_idx = 0;
        let mut end_idx = raw_buffer.len();

        for i in 0..raw_buffer.len() - 1 {
            if raw_buffer[i] < 0.0 && raw_buffer[i + 1] >= 0.0
                || raw_buffer[i] == 0.0 && raw_buffer[i + 1] > 0.0
            {
                raw_buffer[i] = 0.0;
                start_idx = i;
                break;
            }
        }

        for i in (1..raw_buffer.len() - 1).rev() {
            if raw_buffer[i] > 0.0 && raw_buffer[i - 1] <= 0.0
                || raw_buffer[i] == 0.0 && raw_buffer[i - 1] < 0.0
            {
                raw_buffer[i] = 0.0;
                end_idx = i;
                break;
            }
        }

        println!("start {} end {}", start_idx, end_idx);

        raw_buffer = raw_buffer[start_idx..end_idx].to_vec();

        // interpolation samples
        raw_buffer.push(0.0);
        raw_buffer.push(0.0);
        raw_buffer.insert(0, 0.0);
        raw_buffer = stretch_to_size(&raw_buffer, matrix_size.1);

        println!("after {:?}", raw_buffer);
        wavematrix.push(raw_buffer);
    }

    wavematrix
}

pub fn stretch_to_size(buf: &[f32], target_size: usize) -> Vec<f32> {
    let ratio = (buf.len() - 3) as f32 / target_size as f32;

    println!("ratio {}", ratio);

    let mut out_buf = vec![0.0; target_size];
    let mut frac_index: f32 = 1.0;
    let frac_index_increment = ratio;

    for current_sample in out_buf.iter_mut() {
        // get sample:
        let idx = frac_index.floor();
        let frac = frac_index - idx;
        let idx_u = idx as usize;

        // 4-point, 3rd-order Hermite
        *current_sample = interpolate(
            frac,
            buf[idx_u - 1],
            buf[idx_u],
            buf[idx_u + 1],
            buf[idx_u + 2],
            1.0,
        );

        if ((frac_index + frac_index_increment) as usize) < target_size {
            frac_index += frac_index_increment;
        }
    }

    out_buf
}