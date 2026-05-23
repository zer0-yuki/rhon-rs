/// An edit operation in a diff.
///
/// The sequence of operations, when applied in order, transforms `a` into `b`:
/// - [`DiffOp::Keep`]  — the element appears in both sequences (part of the LCS).
/// - [`DiffOp::Delete`] — the element appears only in `a`.
/// - [`DiffOp::Insert`] — the element appears only in `b`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffOp<'a, T> {
    Keep(&'a T),
    Delete(&'a T),
    Insert(&'a T),
}

/// Myers' diff algorithm with backtracking: returns the shortest edit sequence
/// (insertions and deletions only, no substitutions).
///
/// The returned `Vec<DiffOp>` transforms `a` into `b` when applied in order.
///
/// # Complexity
/// - Time:  $O((N+M) \cdot D)$ where $D$ is the edit distance.
/// - Space: $O((N+M) \cdot D)$ due to trace storage for backtracking.
///
/// # Reference
/// Eugene W. Myers, "An O(ND) Difference Algorithm and Its Variations",
/// Algorithmica, 1986.
pub fn diff<'a, T: Eq>(a: &'a [T], b: &'a [T]) -> Vec<DiffOp<'a, T>> {
    let n = a.len() as isize;
    let m = b.len() as isize;
    let max = n + m;

    if max == 0 {
        return vec![];
    }

    let offset = max as usize;
    let mut v = vec![-1isize; 2 * max as usize + 1];
    v[offset + 1] = 0;

    // Trace of V arrays: trace[d] = V at the start of round d
    // (= V after round d-1).  Required for backtracking.
    let mut trace: Vec<Vec<isize>> = Vec::new();
    let mut solution_d = max;

    'outer: for d in 0..=max {
        trace.push(v.clone());

        let mut k = -d;
        while k <= d {
            let idx = (offset as isize + k) as usize;

            let down = if k == -d {
                true
            } else if k == d {
                false
            } else {
                v[idx - 1] < v[idx + 1]
            };

            let k_prev = if down { k + 1 } else { k - 1 };
            let prev_idx = (offset as isize + k_prev) as usize;
            let x_start = v[prev_idx];

            let (mut x, mut y) = if down {
                (x_start, x_start - k_prev + 1)
            } else {
                (x_start + 1, x_start - k_prev)
            };

            // Follow the diagonal snake.
            // Guard y >= 0: in right-moves, y = x_start - k_prev can dip
            // below 0 when x_start < k_prev (virtual region below the grid).
            while x < n && y >= 0 && y < m && a[x as usize] == b[y as usize] {
                x += 1;
                y += 1;
            }

            v[idx] = x;

            if x >= n && y >= m {
                solution_d = d;
                break 'outer;
            }

            k += 2;
        }
    }

    // --- backtrack from (n, m) to (0, 0) using the trace ---
    let mut edits = Vec::new();
    let (mut x, mut y) = (n, m);

    for d in (0..=solution_d).rev() {
        let k = x - y;
        let v_prev = &trace[d as usize];

        // Replay the same decision rule used in the forward pass.
        let down = if k == -d {
            true
        } else if k == d {
            false
        } else {
            let idx_left = (offset as isize + k - 1) as usize;
            let idx_up = (offset as isize + k + 1) as usize;
            v_prev[idx_left] < v_prev[idx_up]
        };

        let k_prev = if down { k + 1 } else { k - 1 };
        let x_prev = v_prev[(offset as isize + k_prev) as usize];

        // The midpoint after the one-step move, before the diagonal snake.
        let x_mid = if down { x_prev } else { x_prev + 1 };

        // Diagonal snake → Keep.
        while x > x_mid {
            x -= 1;
            y -= 1;
            edits.push(DiffOp::Keep(&a[x as usize]));
        }

        // The one step: at d = 0 this would take us to the virtual (0, -1)
        // start state, which is not a real edit.
        if d > 0 {
            if down {
                y -= 1;
                edits.push(DiffOp::Insert(&b[y as usize]));
            } else {
                x -= 1;
                edits.push(DiffOp::Delete(&a[x as usize]));
            }
        }
    }

    edits.reverse();
    edits
}

#[cfg(test)]
mod test {
    use super::*;

    fn keep<'a, T>(r: &'a T) -> DiffOp<'a, T> {
        DiffOp::Keep(r)
    }
    fn delete<'a, T>(r: &'a T) -> DiffOp<'a, T> {
        DiffOp::Delete(r)
    }
    fn insert<'a, T>(r: &'a T) -> DiffOp<'a, T> {
        DiffOp::Insert(r)
    }

    #[test]
    fn identical() {
        let a = b"abc";
        assert_eq!(diff(a, a), vec![keep(&b'a'), keep(&b'b'), keep(&b'c')]);
    }

    #[test]
    fn completely_different() {
        assert_eq!(diff(b"abc", b"def").len(), 6);
    }

    #[test]
    fn one_substitution_like() {
        // "cat" → "cut": delete 'a', keep 'c', insert 'u', keep 't'
        // (one possible shortest edit script)
        let result = diff(b"cat", b"cut");
        assert_eq!(result.len(), 4);
        // Verify the sequence is valid semantically.
        let mut a_pos = 0;
        let mut b_pos = 0;
        for op in &result {
            match op {
                DiffOp::Keep(_) => {
                    a_pos += 1;
                    b_pos += 1;
                }
                DiffOp::Delete(_) => a_pos += 1,
                DiffOp::Insert(_) => b_pos += 1,
            }
        }
        assert_eq!(a_pos, 3);
        assert_eq!(b_pos, 3);
    }

    #[test]
    fn hollow() {
        // "hello"(5) → "hollow"(6): distance = 3
        assert_eq!(diff(b"hello", b"hollow").len(), 5 + 6 - 4); // = 7
    }

    #[test]
    fn empty_vs_nonempty() {
        assert_eq!(
            diff(b"", b"abc"),
            vec![insert(&b'a'), insert(&b'b'), insert(&b'c')]
        );
        assert_eq!(
            diff(b"abc", b""),
            vec![delete(&b'a'), delete(&b'b'), delete(&b'c')]
        );
    }

    #[test]
    fn both_empty() {
        assert_eq!(diff(b"", b""), vec![]);
    }

    #[test]
    fn reconstruct_identity() {
        // Applying the diff to `a` should reconstruct `b`.
        let a = b"axbycz";
        let b = b"pqr";
        let ops = diff(a, b);
        let mut reconstructed = Vec::new();
        for op in ops {
            if let DiffOp::Keep(r) | DiffOp::Insert(r) = op {
                reconstructed.push(*r);
            }
        }
        assert_eq!(reconstructed, b.to_vec());
    }
}
