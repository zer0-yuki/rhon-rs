use std::fmt;

pub fn assert_len_eq<T: fmt::Debug, U: fmt::Debug>(l: &[T], r: &[U]) {
    assert_eq!(
        l.len(),
        r.len(),
        "\nmismatch length:\n  left: (length {}) {:?}\n right: (length {}) {:?}\n",
        l.len(),
        l,
        r.len(),
        r
    );
}
