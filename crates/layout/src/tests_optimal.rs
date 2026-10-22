//! Optimal (Knuth-Plass) line breaking for composition ([`crate::LayoutOptions::optimal_breaks`]).

use super::*;
use wordcraft_doc::props::{Align, ParaProps};
use wordcraft_doc::{Pos, StoryRef};

const PROSE: &str = "The Queen had not moved. Mayne had stopped writing, and for a long moment nobody in the Throne Room said anything at all, because \
the history of Callowen had been written five times and every one of those histories had been wrong in a way that only a person who loved the place could \
see. Cavan Browell stood very still with the manuscript in his hands, thinking about how to say what he had come to say, and about the long hour behind \
him in which he had said everything else. ";

fn doc_of(text: &str, align: Align) -> Document {
    let mut d = Document::from_text(text);
    d.settings.compat_mode = 15;
    d.format_paragraphs(&Pos::body(0, 0), &Pos::body(0, 0), &|p| p.align = Some(align)).unwrap();
    d
}

fn lay_with(doc: &Document, optimal: bool) -> DocLayout {
    let mut c = LayoutCache::new();
    layout(doc, &mut c, &LayoutOptions { optimal_breaks: optimal, ..Default::default() })
}

/// The (start, stop) byte ranges of every line of the first paragraph, and where each line's text ends on the page.
fn lines(l: &DocLayout) -> Vec<(usize, usize, f32, bool)> {
    let mut out = Vec::new();
    for p in &l.pages {
        for it in &p.items {
            if let Placed::Lines { para, l0, l1, x, .. } = it {
                for li in *l0..*l1 {
                    let line = &para.lines[li];
                    let text_end = para.x_of(li, line.stop).map(|e| e + x).unwrap_or(0.0);
                    out.push((line.start, line.stop, text_end, line.hyphen.is_some()));
                }
            }
        }
    }
    out
}

#[test]
fn the_option_is_off_by_default_and_changes_nothing_then() {
    let d = doc_of(&PROSE.repeat(3), Align::Left);
    let mut c = LayoutCache::new();
    let default = lines(&layout(&d, &mut c, &LayoutOptions::default()));
    let off = lines(&lay_with(&d, false));
    assert_eq!(default, off);
}

#[test]
fn optimal_lines_cover_the_paragraph_exactly_once() {
    let d = doc_of(&PROSE.repeat(3), Align::Justify);
    let l = lines(&lay_with(&d, true));
    assert!(l.len() > 5);
    let mut at = 0;
    for (start, stop, _, _) in &l {
        assert_eq!(*start, at, "lines are contiguous: {l:?}");
        assert!(stop > start);
        at = *stop;
    }
    assert_eq!(at, PROSE.repeat(3).len());
}

/// Ragged text: the right edges of the lines are as even as the words allow, so the sum of squared gaps to the margin is no larger than the
/// greedy breaker's, and for this text smaller.
#[test]
fn optimal_breaking_gives_a_more_even_right_edge_than_greedy() {
    let d = doc_of(&PROSE.repeat(4), Align::Left);
    let right = 540.0;
    let raggedness = |l: &[(usize, usize, f32, bool)]| l[..l.len() - 1].iter().map(|(_, _, e, _)| (right - e).powi(2)).sum::<f32>();
    let greedy = lines(&lay_with(&d, false));
    let optimal = lines(&lay_with(&d, true));
    for (_, _, e, hyphen) in &optimal {
        // A trailing space or hyphen hangs just past the margin, as in the greedy layout.
        let _ = hyphen;
        assert!(*e <= right + 6.0, "a line runs past the margin: {e}");
    }
    let (g, o) = (raggedness(&greedy), raggedness(&optimal));
    assert!(o < g, "optimal raggedness {o} must be below greedy {g}");
}

/// Justified text: every line but the last still ends at the margin, and no line is pushed past it.
#[test]
fn optimal_justified_lines_reach_the_margin_and_never_pass_it() {
    let d = doc_of(&PROSE.repeat(4), Align::Justify);
    let l = lines(&lay_with(&d, true));
    // A line's end is measured after its trailing space (or hyphen), which hangs just past the margin.
    for (_, _, e, hyphen) in &l[..l.len() - 1] {
        assert!((539.9..=546.0).contains(e), "a justified line ends at {e} (hyphen: {hyphen})");
    }
    assert!(l.last().unwrap().2 <= 546.0);
}

#[test]
fn a_paragraph_with_a_tab_keeps_the_greedy_breaks() {
    let text = format!("Chapter\tOne. {}", PROSE.repeat(2));
    let d = doc_of(&text, Align::Left);
    assert_eq!(lines(&lay_with(&d, false)), lines(&lay_with(&d, true)), "tabs are not planned: same breaks as without the option");
}

#[test]
fn a_very_long_paragraph_keeps_the_greedy_breaks() {
    let d = doc_of(&PROSE.repeat(12), Align::Left);
    assert!(PROSE.repeat(12).split_whitespace().count() > 500);
    assert_eq!(lines(&lay_with(&d, false)), lines(&lay_with(&d, true)));
}

#[test]
fn the_layout_cache_keeps_the_two_modes_apart() {
    let d = doc_of(&PROSE.repeat(4), Align::Left);
    let mut c = LayoutCache::new();
    let greedy = lines(&layout(&d, &mut c, &LayoutOptions::default()));
    let optimal = lines(&layout(&d, &mut c, &LayoutOptions { optimal_breaks: true, ..Default::default() }));
    let greedy_again = lines(&layout(&d, &mut c, &LayoutOptions::default()));
    assert_ne!(greedy, optimal, "the option takes effect with a warm cache");
    assert_eq!(greedy, greedy_again, "and switching back gives the greedy layout again");
}

#[test]
fn short_paragraphs_and_single_lines_are_unchanged() {
    for text in ["Hi.", "A short line of text.", ""] {
        let d = doc_of(text, Align::Justify);
        assert_eq!(lines(&lay_with(&d, false)), lines(&lay_with(&d, true)), "{text:?}");
    }
}

#[test]
fn first_line_indent_is_respected() {
    let mut d = Document::from_text(&PROSE.repeat(2));
    d.settings.compat_mode = 15;
    let p = d.para_mut(StoryRef::Body, &wordcraft_doc::Path::top(0)).unwrap();
    p.props = ParaProps { align: Some(Align::Left), indent_first: Some(36.0), ..Default::default() };
    let l = lines(&lay_with(&d, true));
    assert!(l.len() > 3);
    assert!(l.iter().all(|(_, _, e, _)| *e <= 546.0));
}
