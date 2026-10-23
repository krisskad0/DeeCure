// SPDX-License-Identifier: Apache-2.0

//! A small line diff, for "what changed in this config file".
//!
//! Config edits are usually a few lines, so the common start and end are cut off first,
//! and only the middle is compared. A middle too big to compare quickly is shown as
//! "all old lines removed, all new lines added", which is still correct.

/// How many unchanged lines to show around each change.
const CONTEXT: usize = 2;
/// The middle part is compared line by line only up to this many line pairs.
const MAX_CELLS: usize = 250_000;

/// A diff, ready to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diff {
    /// Lines starting with `- ` (removed), `+ ` (added), or `  ` (unchanged, for context),
    /// with `…` between parts that are far apart.
    pub text: String,
    /// Lines added.
    pub added: usize,
    /// Lines removed.
    pub removed: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Same,
    Removed,
    Added,
}

/// Compares two texts line by line. The text is cut at `max_bytes` (on a line end).
pub fn lines(old: &str, new: &str, max_bytes: usize) -> Diff {
    let a: Vec<&str> = old.lines().collect();
    let b: Vec<&str> = new.lines().collect();
    let prefix = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let suffix = a[prefix..]
        .iter()
        .rev()
        .zip(b[prefix..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let (mid_a, mid_b) = (&a[prefix..a.len() - suffix], &b[prefix..b.len() - suffix]);

    let mut script: Vec<(Op, &str)> = a[..prefix].iter().map(|l| (Op::Same, *l)).collect();
    script.extend(middle(mid_a, mid_b));
    script.extend(a[a.len() - suffix..].iter().map(|l| (Op::Same, *l)));

    let added = script.iter().filter(|(op, _)| *op == Op::Added).count();
    let removed = script.iter().filter(|(op, _)| *op == Op::Removed).count();
    Diff {
        text: render(&script, max_bytes),
        added,
        removed,
    }
}

/// The edit script for the changed middle part.
fn middle<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<(Op, &'a str)> {
    if a.len().saturating_mul(b.len()) > MAX_CELLS {
        let mut out: Vec<(Op, &str)> = a.iter().map(|l| (Op::Removed, *l)).collect();
        out.extend(b.iter().map(|l| (Op::Added, *l)));
        return out;
    }
    // Longest common subsequence, filled from the end so the walk below goes forward.
    let w = b.len() + 1;
    let mut table = vec![0u32; (a.len() + 1) * w];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            table[i * w + j] = if a[i] == b[j] {
                table[(i + 1) * w + j + 1] + 1
            } else {
                table[(i + 1) * w + j].max(table[i * w + j + 1])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    let mut out = Vec::with_capacity(a.len() + b.len());
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            out.push((Op::Same, a[i]));
            i += 1;
            j += 1;
        } else if table[(i + 1) * w + j] >= table[i * w + j + 1] {
            out.push((Op::Removed, a[i]));
            i += 1;
        } else {
            out.push((Op::Added, b[j]));
            j += 1;
        }
    }
    out.extend(a[i..].iter().map(|l| (Op::Removed, *l)));
    out.extend(b[j..].iter().map(|l| (Op::Added, *l)));
    out
}

/// Changed lines with a little context around them.
fn render(script: &[(Op, &str)], max_bytes: usize) -> String {
    let changed: Vec<usize> = (0..script.len())
        .filter(|&i| script[i].0 != Op::Same)
        .collect();
    let near = |i: usize| {
        changed
            .iter()
            .any(|&c| i + CONTEXT >= c && i <= c + CONTEXT)
    };
    let mut out = String::new();
    let mut last_shown: Option<usize> = None;
    for (i, (op, line)) in script.iter().enumerate() {
        if *op == Op::Same && !near(i) {
            continue;
        }
        if last_shown.is_some_and(|l| i > l + 1) || (last_shown.is_none() && i > 0) {
            out.push_str("…\n");
        }
        let mark = match op {
            Op::Same => "  ",
            Op::Removed => "- ",
            Op::Added => "+ ",
        };
        let next = format!("{mark}{line}\n");
        if out.len() + next.len() > max_bytes {
            out.push_str("… (the rest is not shown)\n");
            return out;
        }
        out.push_str(&next);
        last_shown = Some(i);
    }
    if last_shown.is_some_and(|l| l + 1 < script.len()) {
        out.push_str("…\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_a_changed_line_with_context() {
        let old = "a\nb\nc\nd\ne\nf\ng\n";
        let new = "a\nb\nc\nD\ne\nf\ng\n";
        let d = lines(old, new, 4096);
        assert_eq!((d.added, d.removed), (1, 1));
        assert_eq!(d.text, "…\n  b\n  c\n- d\n+ D\n  e\n  f\n…\n");
    }

    #[test]
    fn added_and_removed_files_and_far_apart_changes() {
        let d = lines("", "x = 1\ny = 2\n", 4096);
        assert_eq!((d.added, d.removed), (2, 0));
        assert_eq!(d.text, "+ x = 1\n+ y = 2\n");

        let old: String = (0..20).map(|i| format!("{i}\n")).collect();
        let new: String = (0..20)
            .map(|i| match i {
                2 => "two\n".to_owned(),
                17 => "seventeen\n".to_owned(),
                _ => format!("{i}\n"),
            })
            .collect();
        let d = lines(&old, &new, 4096);
        assert_eq!((d.added, d.removed), (2, 2));
        assert_eq!(
            d.text.matches('…').count(),
            1,
            "one gap between the two changes: {}",
            d.text
        );
    }

    #[test]
    fn inserted_lines_keep_the_rest_unchanged() {
        let d = lines("a\nb\nc\n", "a\nnew\nb\nc\n", 4096);
        assert_eq!((d.added, d.removed), (1, 0));
        assert_eq!(d.text, "  a\n+ new\n  b\n  c\n");
    }

    #[test]
    fn huge_changes_fall_back_and_long_output_is_cut() {
        let old: String = (0..1000).map(|i| format!("old {i}\n")).collect();
        let new: String = (0..1000).map(|i| format!("new {i}\n")).collect();
        let d = lines(&old, &new, 200);
        assert_eq!((d.added, d.removed), (1000, 1000));
        assert!(d.text.len() <= 230, "{}", d.text.len());
        assert!(d.text.ends_with("… (the rest is not shown)\n"));
    }

    #[test]
    fn identical_texts_have_no_diff() {
        let d = lines("same\n", "same\n", 100);
        assert_eq!((d.added, d.removed, d.text.as_str()), (0, 0, ""));
    }
}
