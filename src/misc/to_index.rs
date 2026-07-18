pub fn to_index(idx: f64, len: usize) -> usize {
    if idx as i32 >= 0 {
        if idx < len as f64 {
            return idx as usize;
        } else {
            panic!("OutOfBoundsError: Index is {idx} but len is {len}.")
        }
    }

    return len - idx as usize - 1;
}
