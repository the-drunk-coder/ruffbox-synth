use crate::building_blocks::interpolation::interpolate;

/// stretch (interpolate) buffer to target size
pub fn stretch_to_size(buf: &[f32], target_size: usize) -> Vec<f32> {
    let ratio = (buf.len() - 3) as f32 / target_size as f32;

    //println!("ratio {}", ratio);

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

/// find the next zero crossing on beginning and end of the buffer
pub fn find_zerocrossings(buffer: &[f32], reverse: bool) -> (usize, usize, usize) {
    // find zerocrossing from new to pos
    let mut start_idx = 0;
    let mut end_idx = buffer.len();

    if reverse {
        for i in 0..buffer.len() - 1 {
            if buffer[i] > 0.0 && buffer[i + 1] <= 0.0 || buffer[i] == 0.0 && buffer[i + 1] < 0.0 {
                start_idx = i;
                break;
            }
        }

        for i in (1..buffer.len() - 1).rev() {
            if buffer[i] < 0.0 && buffer[i - 1] >= 0.0 || buffer[i] == 0.0 && buffer[i - 1] > 0.0 {
                end_idx = i;
                break;
            }
        }
    } else {
        for i in 0..buffer.len() - 1 {
            if buffer[i] < 0.0 && buffer[i + 1] >= 0.0 || buffer[i] == 0.0 && buffer[i + 1] > 0.0 {
                start_idx = i;
                break;
            }
        }

        for i in (1..buffer.len() - 1).rev() {
            if buffer[i] > 0.0 && buffer[i - 1] <= 0.0 || buffer[i] == 0.0 && buffer[i - 1] < 0.0 {
                end_idx = i;
                break;
            }
        }
    }

    (start_idx, end_idx + 1, end_idx + 1 - start_idx)
}
