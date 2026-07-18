pub fn to_index(idx: f64, len: usize) -> usize {
    if idx >= 0.0 {
        let idx = idx as usize;
        assert!(idx < len, "OutOfBoundsError");
        idx
    } else {
        let idx = (-idx) as usize;
        assert!(idx <= len, "OutOfBoundsError");
        len - idx
    }
}
