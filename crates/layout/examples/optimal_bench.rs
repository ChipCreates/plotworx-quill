//! Cost of optimal line breaking: lay out a manuscript-sized document with and without `LayoutOptions::optimal_breaks`.
//! Usage: cargo run --release -p wordcraft-layout --example optimal_bench -- [paragraphs] [words per paragraph]

use std::time::Instant;
use wordcraft_doc::{Document, Paragraph, para_block};
use wordcraft_layout::{LayoutCache, LayoutOptions, layout};

const WORDS: &[&str] = &[
    "the",
    "queen",
    "had",
    "not",
    "moved",
    "mayne",
    "stopped",
    "writing",
    "history",
    "of",
    "callowen",
    "because",
    "he",
    "did",
    "know",
    "what",
    "was",
    "said",
    "in",
    "throne",
    "room",
    "fifteenth",
    "harvestend",
    "archive",
    "manuscript",
    "oath",
    "requires",
    "objective",
    "disappear",
    "silence",
    "listening",
    "extraordinarily",
    "understanding",
    "remembered",
    "everything",
    "afternoon",
    "considered",
    "impossible",
    "whispered",
    "unfinished",
];

fn main() {
    let args: Vec<usize> = std::env::args().skip(1).filter_map(|a| a.parse().ok()).collect();
    let (paras, words) = (args.first().copied().unwrap_or(800), args.get(1).copied().unwrap_or(120));
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let mut doc = Document::new();
    doc.settings.compat_mode = 15;
    let mut blocks = Vec::new();
    for _ in 0..paras {
        let n = words / 2 + (next() as usize % words);
        let text: Vec<&str> = (0..n).map(|_| WORDS[next() as usize % WORDS.len()]).collect();
        let mut p = Paragraph::with_text(&text.join(" "), Default::default());
        p.props.align = Some(wordcraft_doc::props::Align::Justify);
        blocks.push(para_block(p));
    }
    doc.body = blocks;
    let total_words = paras * words;
    let breaks = |optimal: bool| -> Vec<Vec<usize>> {
        let mut cache = LayoutCache::new();
        let l = layout(&doc, &mut cache, &LayoutOptions { optimal_breaks: optimal, ..Default::default() });
        let mut out: Vec<Vec<usize>> = Vec::new();
        for page in &l.pages {
            for it in &page.items {
                if let wordcraft_layout::Placed::Lines { para, l0, l1, .. } = it {
                    out.push(para.lines[*l0..*l1].iter().map(|x| x.start).collect());
                }
            }
        }
        out
    };
    let (g, o) = (breaks(false), breaks(true));
    let differ = g.iter().zip(&o).filter(|(a, b)| a != b).count();
    println!(
        "BENCH|paragraph pieces {} greedy, {} optimal; {} differ in where lines break; lines: greedy {}, optimal {}",
        g.len(),
        o.len(),
        differ,
        g.iter().map(Vec::len).sum::<usize>(),
        o.iter().map(Vec::len).sum::<usize>()
    );
    for (label, optimal) in [("greedy", false), ("optimal", true), ("greedy", false), ("optimal", true)] {
        let mut cache = LayoutCache::new();
        let t = Instant::now();
        let l = layout(&doc, &mut cache, &LayoutOptions { optimal_breaks: optimal, ..Default::default() });
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        println!(
            "BENCH|{label:8}|cold layout of ~{total_words} words ({} paragraphs, {} pages): {ms:.0} ms ({:.3} ms per paragraph)",
            paras,
            l.pages.len(),
            ms / paras as f64
        );
    }
    // Re-layout after one edit with a warm cache: only the edited paragraph is laid out again.
    for (label, optimal) in [("greedy", false), ("optimal", true)] {
        let mut cache = LayoutCache::new();
        let opts = LayoutOptions { optimal_breaks: optimal, ..Default::default() };
        let _ = layout(&doc, &mut cache, &opts);
        let mut d2 = doc.clone();
        if let Some(wordcraft_doc::Block::Para(p)) = d2.body.get(paras / 2).map(|b| (**b).clone()) {
            let mut p = p;
            p.insert_text(5, "edit ", &Default::default()).ok();
            d2.body[paras / 2] = para_block(p);
        }
        let t = Instant::now();
        let _ = layout(&d2, &mut cache, &opts);
        println!("BENCH|{label:8}|warm re-layout after one edit: {:.1} ms", t.elapsed().as_secs_f64() * 1000.0);
    }
}
