use crate::helpers::misc::*;

pub enum WavetableizeMethod {
    Raw,
    ZerocrossingFixedRangeStretchInverse,
}

pub fn wavetableize(
    buffer: &[f32],
    matrix_size: (usize, usize),
    start: f32,
    method: WavetableizeMethod,
) -> Vec<Vec<f32>> {
    match method {
        WavetableizeMethod::Raw => raw(buffer, matrix_size, start),
        WavetableizeMethod::ZerocrossingFixedRangeStretchInverse => {
            zerocrossing_fixed_range_stretch_inverse(buffer, matrix_size, start)
        }
    }
}

/// first, naive implementation to chop a sample buffer to a wavetable
fn raw(buffer: &[f32], mut matrix_size: (usize, usize), start: f32) -> Vec<Vec<f32>> {
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

fn zerocrossing_fixed_range_stretch_inverse(
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

        let zc_reg = find_zerocrossings(&raw_buffer, false);
        let zc_inv = find_zerocrossings(&raw_buffer, true);

        let (inverse, start_idx, end_idx) = if zc_inv.2 > zc_reg.2 {
            (true, zc_inv.0, zc_reg.1)
        } else {
            (true, zc_reg.0, zc_reg.1)
        };

        //println!("start {} end {} {}", start_idx, end_idx, inverse);

        raw_buffer[start_idx] = 0.0;
        raw_buffer[end_idx - 1] = 0.0;
        raw_buffer = raw_buffer[start_idx..end_idx].to_vec();

        //inverse phase
        if inverse {
            raw_buffer = raw_buffer.iter_mut().map(|x| *x * -1.0).collect();
        }

        // interpolation samples
        raw_buffer.push(0.0);
        raw_buffer.push(0.0);
        raw_buffer.insert(0, 0.0);
        raw_buffer = stretch_to_size(&raw_buffer, matrix_size.1);

        //println!("after {:?}", raw_buffer);
        wavematrix.push(raw_buffer);
    }

    wavematrix
}
