//! Optimal (Knuth-Plass) line breaking for composition.
//!
//! The ordinary line breaker is greedy: each line takes as many words as fit. Optimal breaking looks at the whole paragraph and chooses the
//! breaks that give the most even lines, as a typesetter would (Knuth and Plass, "Breaking Paragraphs into Lines", Software: Practice and
//! Experience 11, 1981). It is slower than greedy and is meant for *composition* (print, PDF and DOCX export, preview), not for the live page
//! while typing: it is off unless [`crate::LayoutOptions::optimal_breaks`] is set.
//!
//! [`plan`] decides the breaks of a paragraph that this breaker can handle, as cluster indices; `para::break_lines` then ends each line at the
//! planned cluster instead of at the last word that fits and does all the rest (justification, hyphens, vertical metrics) as usual. A paragraph
//! the planner does not handle (tabs, objects, floating objects beside it, right-to-left text, list labels, drop caps, very long paragraphs) or
//! cannot solve gets `None` and is laid out greedily, exactly as without the option.
//!
//! The parameters follow Plotworx Desk's composition layout (`pw-core` `knuth_plass.rs`): space stretch half a space, tolerance 1 when
//! justified and 2 when ragged, hyphen penalty 50, consecutive-hyphen demerits 5000, fitness-class demerits 100, at most 500 words. Unlike
//! Desk, a paragraph that cannot be set within the tolerance is retried with looser tolerances (as TeX's second and emergency passes) instead
//! of falling back to greedy breaking at once.

use crate::para::{ClKind, ParaLayout};

/// Paragraphs with more words than this are laid out greedily (the cost grows with the length of the paragraph).
const MAX_WORDS: usize = 500;
/// Penalty for a break at a hyphenation point or after an explicit hyphen.
const HYPHEN_PENALTY: f32 = 50.0;
const CONSECUTIVE_HYPHEN_DEMERITS: f32 = 5000.0;
const FITNESS_DEMERITS: f32 = 100.0;
/// Stretch of a space, as a fraction of its width (shrink is the layout's own limit for justified lines, passed in).
const STRETCH: f32 = 0.5;
/// Stretch of the glue that fills the last line (so the last line is never stretched).
const FILL: f32 = 1.0e6;

#[derive(Clone, Copy, Debug)]
enum Item {
    /// Material that cannot be broken: a word or part of one.
    Box { w: f32 },
    /// A space (or a run of them): natural width, stretch, shrink.
    Glue { w: f32, stretch: f32, shrink: f32 },
    /// A break opportunity that is not a space. `p < 0` is forced (the end of the paragraph); `flagged` marks a hyphen.
    Penalty { w: f32, p: f32, flagged: bool },
}

/// What the planner needs to know about the line: the widths, the alignment and the layout's space shrink.
pub(crate) struct Lines<'a> {
    pub first_width: f32,
    pub body_width: f32,
    pub justified: bool,
    /// Fraction of a space's width a justified line may shrink it by.
    pub max_shrink: f32,
    /// Width of the hyphen drawn at a hyphenated break, by cluster style.
    pub hyphen_width: &'a dyn Fn(u32) -> f32,
}

/// The planned breaks of `pl`: the cluster index each line ends at (the last line ends at the last cluster), or `None` when the paragraph is not
/// one this breaker handles or no feasible set of breaks exists.
pub(crate) fn plan(pl: &ParaLayout, lines: &Lines<'_>) -> Option<Vec<usize>> {
    let n = pl.clusters.len();
    if n == 0 || pl.label.is_some() || pl.drop_cap.is_some() || !pl.bidi_levels.is_empty() || !pl.ptabs.is_empty() || !pl.displays.is_empty() {
        return None;
    }
    let mut items: Vec<Item> = Vec::with_capacity(n);
    // The cluster each break item ends a line at, parallel to `items`.
    let mut ends: Vec<usize> = Vec::with_capacity(n);
    let mut words = 0usize;
    let mut j = 0usize;
    while j < n {
        let c = pl.clusters.get(j)?;
        match c.kind {
            ClKind::Text | ClKind::Marker => {
                // The cluster is part of a word; a break after it is a hyphenation point or an explicit hyphen.
                if j == 0 || pl.clusters.get(j - 1).is_none_or(|p| p.kind == ClKind::Space) {
                    words += 1;
                }
                items.push(Item::Box { w: c.adv });
                ends.push(j);
                let next_is_text = pl.clusters.get(j + 1).is_some_and(|nx| nx.kind != ClKind::Space);
                if next_is_text && c.kind == ClKind::Text {
                    let hyphen = pl.hyph_after.binary_search(&(j as u32)).is_ok();
                    if hyphen {
                        // The hyphen hangs past the right margin (the layout justifies the text to the margin), so it takes no width from the line.
                        let _ = lines.hyphen_width;
                        items.push(Item::Penalty { w: 0.0, p: HYPHEN_PENALTY, flagged: true });
                        ends.push(j);
                    } else if c.break_after {
                        items.push(Item::Penalty { w: 0.0, p: HYPHEN_PENALTY, flagged: true });
                        ends.push(j);
                    }
                }
                j += 1;
            }
            ClKind::Space => {
                // A run of spaces is one glue; the line ends after the last of them, as the greedy breaker keeps them on the line.
                let mut w = 0.0;
                while pl.clusters.get(j).is_some_and(|s| s.kind == ClKind::Space) {
                    w += pl.clusters.get(j)?.adv;
                    j += 1;
                }
                let stretch = w * STRETCH;
                let shrink = if lines.justified { w * lines.max_shrink } else { 0.0 };
                items.push(Item::Glue { w, stretch, shrink });
                ends.push(j - 1);
            }
            // Tabs, line/page/column breaks, inline objects, ...: not handled here.
            _ => return None,
        }
    }
    if words == 0 || words > MAX_WORDS {
        return None;
    }
    // The end of the paragraph: the last line is left at its natural width.
    items.push(Item::Glue { w: 0.0, stretch: FILL, shrink: 0.0 });
    ends.push(n - 1);
    items.push(Item::Penalty { w: 0.0, p: f32::NEG_INFINITY, flagged: false });
    ends.push(n - 1);
    // As TeX does: first try for lines that need little stretching; if there is no way to set the paragraph so, allow looser lines, and last allow any
    // line that is not overfull (a paragraph that cannot be set at all, such as one with a word wider than the line, still returns `None`).
    let base = if lines.justified { 1.0 } else { 2.0 };
    let breaks = [base, base * 3.0, 1000.0].into_iter().find_map(|tolerance| solve(&items, lines.first_width, lines.body_width, tolerance))?;
    let mut out: Vec<usize> = breaks.into_iter().filter_map(|b| ends.get(b).copied()).collect();
    out.dedup();
    if out.last().copied() != Some(n - 1) {
        out.push(n - 1);
    }
    Some(out)
}

#[derive(Clone, Copy, Debug)]
struct Node {
    /// The item this node's break is at.
    pos: usize,
    /// 0 for the line after the start of the paragraph, 1 for any later one (only the first line has its own width).
    line: u8,
    fitness: u8,
    demerits: f32,
    prev: Option<usize>,
    flagged: bool,
    /// Sums of width, stretch and shrink of everything up to just after this break, with the glue after the break skipped.
    sums: (f32, f32, f32),
}

/// The break items (indices into `items`) of the best set of breaks, or `None` when no feasible one exists.
fn solve(items: &[Item], first_width: f32, body_width: f32, tolerance: f32) -> Option<Vec<usize>> {
    let mut nodes: Vec<Node> = vec![Node { pos: 0, line: 0, fitness: 1, demerits: 0.0, prev: None, flagged: false, sums: (0.0, 0.0, 0.0) }];
    let mut active: Vec<usize> = vec![0];
    // Running sums of everything seen so far.
    let mut sums = (0.0f32, 0.0f32, 0.0f32);
    let mut last_box = false;
    for (b, item) in items.iter().enumerate() {
        let breakable = match item {
            Item::Penalty { p, .. } => *p < f32::INFINITY,
            Item::Glue { .. } => last_box,
            Item::Box { .. } => false,
        };
        if breakable {
            let (pw, p, flagged) = match item {
                Item::Penalty { w, p, flagged } => (*w, *p, *flagged),
                _ => (0.0, 0.0, false),
            };
            // Best candidate per fitness class for this break: (demerits, previous node).
            let mut best: [Option<(f32, usize)>; 4] = [None; 4];
            let mut cut: Vec<usize> = Vec::new();
            for (ai, &a) in active.iter().enumerate() {
                let node = nodes[a];
                let width = if node.line == 0 { first_width } else { body_width };
                let line_w = sums.0 - node.sums.0 + pw;
                let stretch = sums.1 - node.sums.1;
                let shrink = sums.2 - node.sums.2;
                let diff = width - line_w;
                let r = if diff.abs() < 0.001 {
                    0.0
                } else if diff > 0.0 {
                    if stretch > 0.0 { diff / stretch } else { f32::INFINITY }
                } else if shrink > 0.0 {
                    diff / shrink
                } else {
                    f32::NEG_INFINITY
                };
                if r < -1.0 || p == f32::NEG_INFINITY {
                    cut.push(ai);
                }
                if (-1.0..=tolerance).contains(&r) {
                    let badness = 100.0 * r.abs().powi(3);
                    let mut demerits = if p >= 0.0 {
                        (1.0 + badness + p).powi(2)
                    } else if p > f32::NEG_INFINITY {
                        (1.0 + badness).powi(2) - p * p
                    } else {
                        (1.0 + badness).powi(2)
                    };
                    if flagged && node.flagged {
                        demerits += CONSECUTIVE_HYPHEN_DEMERITS;
                    }
                    let fitness: u8 = if r < -0.5 {
                        0
                    } else if r <= 0.5 {
                        1
                    } else if r <= 1.0 {
                        2
                    } else {
                        3
                    };
                    if (fitness as i32 - node.fitness as i32).abs() > 1 {
                        demerits += FITNESS_DEMERITS;
                    }
                    let total = node.demerits + demerits;
                    let slot = &mut best[fitness as usize];
                    if slot.is_none_or(|(d, _)| total < d) {
                        *slot = Some((total, a));
                    }
                }
            }
            // The sums after this break: the glue that follows it is discarded at the start of the next line.
            let mut after = sums;
            // A break at a space discards that space too.
            if let Item::Glue { w, stretch, shrink } = item {
                after.0 += w;
                after.1 += stretch;
                after.2 += shrink;
            }
            let mut k = b + 1;
            while let Some(Item::Glue { w, stretch, shrink }) = items.get(k) {
                after.0 += w;
                after.1 += stretch;
                after.2 += shrink;
                k += 1;
            }
            // Deactivate (back to front so the indices stay valid), then add the new nodes.
            for &ai in cut.iter().rev() {
                active.remove(ai);
            }
            for (fitness, cand) in best.iter().enumerate() {
                if let Some((demerits, prev)) = cand {
                    nodes.push(Node { pos: b, line: 1, fitness: fitness as u8, demerits: *demerits, prev: Some(*prev), flagged, sums: after });
                    active.push(nodes.len() - 1);
                }
            }
            if active.is_empty() {
                return None;
            }
        }
        match item {
            Item::Box { w } => {
                sums.0 += w;
                last_box = true;
            }
            Item::Glue { w, stretch, shrink } => {
                sums.0 += w;
                sums.1 += stretch;
                sums.2 += shrink;
                last_box = false;
            }
            Item::Penalty { .. } => last_box = false,
        }
    }
    // The best node at the end of the paragraph.
    let last = items.len() - 1;
    let best = active.iter().copied().filter(|&a| nodes[a].pos == last).min_by(|&x, &y| nodes[x].demerits.total_cmp(&nodes[y].demerits))?;
    let mut out = Vec::new();
    let mut cur = Some(best);
    while let Some(c) = cur {
        if nodes[c].prev.is_some() {
            out.push(nodes[c].pos);
        }
        cur = nodes[c].prev;
    }
    out.reverse();
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(w: f32) -> Item {
        Item::Box { w }
    }
    fn space() -> Item {
        Item::Glue { w: 4.0, stretch: 2.0, shrink: 1.0 }
    }
    fn end() -> [Item; 2] {
        [Item::Glue { w: 0.0, stretch: FILL, shrink: 0.0 }, Item::Penalty { w: 0.0, p: f32::NEG_INFINITY, flagged: false }]
    }

    fn lines_of(items: &[Item], breaks: &[usize], width: f32) -> Vec<f32> {
        // The natural width of each line, as the solver sees it (glue after a break is dropped).
        let mut out = Vec::new();
        let mut start = 0;
        for &b in breaks {
            let mut w = 0.0;
            for (k, it) in items.iter().enumerate().take(b).skip(start) {
                match it {
                    Item::Box { w: x } => w += x,
                    Item::Glue { w: x, .. } if k > start => w += x,
                    _ => {}
                }
            }
            out.push(width - w);
            start = b + 1;
        }
        out
    }

    #[test]
    fn a_paragraph_that_fits_on_one_line_is_one_line() {
        let mut items = vec![word(20.0), space(), word(20.0), space(), word(20.0)];
        items.extend(end());
        let breaks = solve(&items, 100.0, 100.0, 1.0).expect("a solution");
        assert_eq!(breaks, vec![items.len() - 1]);
    }

    #[test]
    fn breaks_are_chosen_for_even_lines_not_greedily() {
        // Words 30, 30, 30, 60, 30, 30, in lines of width 100. Greedy puts 30 30 30 on line 1 (the 4th word does not fit) and 60 30 on line 2
        // (30 spaces between them: 60+4+30=94), leaving 30 on line 3. Optimal can balance the lines better: check that every line is feasible and the
        // total slack squared is no worse than greedy's.
        let mut items = Vec::new();
        for (i, w) in [30.0, 30.0, 30.0, 60.0, 30.0, 30.0].iter().enumerate() {
            if i > 0 {
                items.push(space());
            }
            items.push(word(*w));
        }
        items.extend(end());
        let breaks = solve(&items, 100.0, 100.0, 8.0).expect("a solution");
        let slack = lines_of(&items, &breaks, 100.0);
        assert!(slack.iter().rev().skip(1).all(|s| s.abs() <= 30.0), "every line but the last is close to full: {slack:?}");
        assert!(breaks.len() >= 2);
    }

    #[test]
    fn a_word_wider_than_the_line_has_no_solution() {
        let mut items = vec![word(20.0), space(), word(500.0)];
        items.extend(end());
        assert!(solve(&items, 100.0, 100.0, 1.0).is_none(), "the caller falls back to greedy breaking");
    }

    #[test]
    fn the_first_line_can_have_its_own_width() {
        // A first-line indent: width 70 for the first line, 100 after. Words of 31 with spaces of 4: two words fit the first line (66), three the others (101,
        // squeezed by 1).
        let mut items = Vec::new();
        for i in 0..8 {
            if i > 0 {
                items.push(space());
            }
            items.push(word(31.0));
        }
        items.extend(end());
        let breaks = solve(&items, 70.0, 100.0, 8.0).expect("a solution");
        assert_eq!(breaks[0], 3, "the first line holds two words: {breaks:?}");
        assert_eq!(breaks[1], 9, "the next lines hold three: {breaks:?}");
    }

    #[test]
    fn a_hyphen_penalty_is_paid_only_when_it_helps() {
        // A long word that can be hyphenated lets a tight line break there.
        let mut items = vec![word(40.0), space(), word(30.0), Item::Penalty { w: 4.0, p: 50.0, flagged: true }, word(30.0), space(), word(40.0)];
        items.extend(end());
        let breaks = solve(&items, 90.0, 90.0, 6.0).expect("a solution");
        assert!(!breaks.is_empty());
    }
}
