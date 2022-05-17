use crate::helpers::misc::*;

pub enum WavetableizeMethod {
    Raw,
    //ZerocrossingFixedRangeStretchInverse,
    //Crossfade,
    Smooth,
    Supersmooth,
}

pub fn wavetableize(
    buffer: &[f32],
    matrix_size: (usize, usize),
    start: f32,
    method: WavetableizeMethod,
) -> Vec<Vec<f32>> {
    match method {
        WavetableizeMethod::Raw => raw(buffer, matrix_size, start),
        WavetableizeMethod::Smooth => smooth(buffer, matrix_size, start),
        WavetableizeMethod::Supersmooth => supersmooth(buffer, matrix_size, start),
        //WavetableizeMethod::Crossfade => crossfade(buffer, matrix_size, start),
        //WavetableizeMethod::ZerocrossingFixedRangeStretchInverse => {
        //    zerocrossing_fixed_range_stretch_inverse(buffer, matrix_size, start)
        //}
    }
}

/// first, naive implementation to chop a sample buffer to a wavetable
/// no interpolation, so clicks will be likely
fn raw(buffer: &[f32], matrix_size: (usize, usize), start: f32) -> Vec<Vec<f32>> {
    // calcluate max step size ...
    let remaining_length = buffer.len() - (buffer.len() as f32 * start) as usize;
    let step_size = if remaining_length / matrix_size.1 < matrix_size.0 {
        remaining_length / matrix_size.0 as usize
    } else {
        matrix_size.1
    };

    let mut wavematrix = Vec::new();

    let start: usize = (buffer.len() as f32 * start) as usize;

    for i in 0..matrix_size.0 {
        let offset = start + i * step_size;
        wavematrix.push(buffer[offset..offset + matrix_size.1].to_vec());
    }

    wavematrix
}

/// smoothing filter ... pseudo-gaussian ...
fn smooth(buffer: &[f32], matrix_size: (usize, usize), start: f32) -> Vec<Vec<f32>> {
    // calcluate max step size ...
    let remaining_length = buffer.len() - (buffer.len() as f32 * start) as usize;
    let step_size = if remaining_length / matrix_size.1 < matrix_size.0 {
        remaining_length / matrix_size.0 as usize
    } else {
        matrix_size.1
    };

    let mut wavematrix = Vec::new();

    let start: usize = (buffer.len() as f32 * start) as usize;

    for i in 0..matrix_size.0 {
        let offset = start + i * step_size;
        let mut buf = buffer[offset..offset + matrix_size.1].to_vec();

        // append copy of itself
        buf.append(&mut buf.clone());

        // 16-point smooting window
        let center = buf.len() / 2;
        let smooth_start = center - 8;

        // 7-point triangular smoothing ...
        for i in smooth_start..smooth_start + 16 {
            buf[i] = (buf[i - 3]
                + 2.0 * buf[i - 2]
                + 3.0 * buf[i - 1]
                + 4.0 * buf[i]
                + 3.0 * buf[i + 1]
                + 2.0 * buf[i + 2]
                + buf[i + 3])
                / 16.0;
        }

        for i in 0..8 {
            buf[i] = buf[center + i];
        }

        buf.truncate(center);

        wavematrix.push(buf);
    }

    wavematrix
}

/*
fn crossfade(buffer: &[f32], mut matrix_size: (usize, usize), start_factor: f32) -> Vec<Vec<f32>> {
    // check if final size is possible,
    // correct if necessary
    if buffer.len() / matrix_size.1 < matrix_size.0 {
        matrix_size.0 = buffer.len() / matrix_size.1;
    }

    let offset = matrix_size.1 / 2;

    let mut wavematrix = Vec::new();

    let start: usize = (buffer.len() as f32 * start_factor) as usize;

    for i in 0..matrix_size.0 {
        let temp_buf_1 =
            buffer[(start + i * matrix_size.1)..(start + ((i + 1) * matrix_size.1))].to_vec();
        let mut temp_buf_2 = buffer
            [(start + offset + i * matrix_size.1)..(start + offset + ((i + 1) * matrix_size.1))]
            .to_vec();

        for i in 0..(matrix_size.1 / 2) {
            let fade_in = ((std::f32::consts::PI / 4.0) / (matrix_size.1 / 2) as f32).sin();
            let fade_out = ((std::f32::consts::PI / 4.0) / (matrix_size.1 / 2) as f32).cos();
            temp_buf_2[i + (matrix_size.1 / 2)] =
                fade_in * temp_buf_1[i] + fade_out * temp_buf_2[i + (matrix_size.1 / 2)];
        }

        wavematrix.push(temp_buf_2);
    }

    wavematrix
}*/

fn supersmooth(buffer: &[f32], matrix_size: (usize, usize), start_factor: f32) -> Vec<Vec<f32>> {
    let remaining_length = buffer.len() - (buffer.len() as f32 * start_factor) as usize;
    let step_size = if remaining_length / matrix_size.1 < matrix_size.0 {
        remaining_length / matrix_size.0 as usize
    } else {
        matrix_size.1
    };

    let mut wavematrix = Vec::new();

    let start: usize = (buffer.len() as f32 * start_factor) as usize;

    for i in 0..matrix_size.0 {
        let begin_idx = find_closest_upward_zerocrossing(buffer, start + i * step_size);
        let end_idx =
            find_closest_downward_zerocrossing(buffer, start + i * step_size + matrix_size.1);

        let begin_idx_inv = find_closest_downward_zerocrossing(buffer, start + i * step_size);
        let end_idx_inv =
            find_closest_upward_zerocrossing(buffer, start + i * step_size + matrix_size.1);

        println!(
            "inv b {} inv e {} diff {}",
            begin_idx_inv,
            end_idx_inv,
            ((end_idx_inv - begin_idx_inv) as i32 - matrix_size.1 as i32).abs()
        );
        println!(
            "reg b {} reg e {} diff {}",
            begin_idx,
            end_idx,
            ((end_idx - begin_idx) as i32 - matrix_size.1 as i32).abs()
        );

        let mut buf = if ((end_idx - begin_idx) as i32 - matrix_size.1 as i32).abs()
            > ((end_idx_inv - begin_idx_inv) as i32 - matrix_size.1 as i32).abs()
        {
            println!("choose inv");
            let mut buf = buffer[begin_idx_inv..end_idx_inv + 1].to_vec();
            println!("buf {} {:?}", buf.len(), buf);
            buf = buf.iter_mut().map(|x| *x * -1.0).collect();
            println!("buf inv {} {:?}", buf.len(), buf);
            buf
        } else {
            println!("choose reg");
            buffer[begin_idx..end_idx + 1].to_vec()
        };

        println!("buf {} {:?}", buf.len(), buf);

        buf[0] = 0.0;

        // interpolation samples
        buf.push(0.0);
        buf.push(0.0);
        buf.insert(0, 0.0);
        buf = stretch_to_size(&buf, matrix_size.1);

        println!("buf after stretch {} {:?}", buf.len(), buf);

        /*
            // append copy of itself
            buf.append(&mut buf.clone());

            // 16-point smooting window
            let center = buf.len() / 2;

            let mut smooth_start = center - 4;

            // 5-point triangular smoothing ...
            for i in smooth_start..smooth_start + 8 {
                buf[i] = (buf[i - 2]
                          + 2.0 * buf[i - 1]
                          + 3.0 * buf[i]
                          + 2.0 * buf[i + 1]
                          + buf[i + 2]) / 9.0;
            }

        smooth_start = center - 8;

            // 7-point triangular smoothing ...
            for i in smooth_start..smooth_start + 16 {
                buf[i] = (buf[i - 3]
                    + 2.0 * buf[i - 2]
                    + 3.0 * buf[i - 1]
                    + 4.0 * buf[i]
                    + 3.0 * buf[i + 1]
                    + 2.0 * buf[i + 2]
                    + buf[i + 3])
                    / 16.0;
            }

        smooth_start = center - 16;

            // 9-point triangular smoothing ...
            for i in smooth_start..smooth_start + 32 {
                buf[i] = (buf[i - 4]
                  + 2.0 * buf[i - 3]
                          + 3.0 * buf[i - 2]
                          + 4.0 * buf[i - 1]
                          + 5.0 * buf[i]
                          + 4.0 * buf[i + 1]
                          + 3.0 * buf[i + 2]
                          + 2.0 * buf[i + 3]
                  + buf[i + 4])
                    / 25.0;
            }

            for i in 0..16 {
                buf[i] = buf[center + i];
            }

        buf.truncate(center);
        buf[0] = 0.0;
             */

        println!("final buf {} {:?}", buf.len(), buf);

        wavematrix.push(buf);
    }
    wavematrix
}

/*
/// this one is a bit overcomplicated ...
fn zerocrossing_fixed_range_stretch_inverse(
    buffer: &[f32],
    matrix_size: (usize, usize),
    start: f32,
) -> Vec<Vec<f32>> {
    // check if final size is possible,
    // correct if necessary
    // calcluate max step size ...
    let remaining_length = buffer.len() - (buffer.len() as f32 * start) as usize;
    let step_size = if remaining_length / matrix_size.1 < matrix_size.0 {
        remaining_length / matrix_size.0 as usize
    } else {
        matrix_size.1
    };

    let mut wavematrix = Vec::new();

    let start: usize = (buffer.len() as f32 * start) as usize;

    for i in 0..(matrix_size.0 - 1) {
        let mut raw_buffer =
            buffer[(start + i * step_size)..((start + i * step_size) + matrix_size.1)].to_vec();

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
}*/
