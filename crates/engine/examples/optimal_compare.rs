//! Side-by-side of greedy and optimal (Knuth-Plass) line breaking on real scenes.
//!
//! Usage: cargo run --release -p wordcraft-engine --example optimal_compare -- <scene.md>... --out <dir> [--pages N]
//! For each scene: justify the body, hyphenate, lay out both ways, print line statistics (how far the spaces of justified lines stretch or shrink
//! from their natural width), time both, and write `<scene>-greedy-pN.png` and `<scene>-optimal-pN.png` for the first pages.

use std::time::Instant;
use wordcraft_doc::{Pos, props::Align};
use wordcraft_layout::{LayoutCache, LayoutOptions, Placed, layout};
use wordcraft_render::{RenderOptions, render_page};

struct Stats {
    lines: usize,
    justified_lines: usize,
    /// Mean and worst of |space width / natural space width - 1| over justified lines.
    mean_dev: f64,
    worst_dev: f64,
    /// Lines whose spaces are stretched or shrunk by more than 50%.
    loose: usize,
    hyphens: usize,
    /// Knuth-Plass badness of the justified lines (100 |r|^3, r = the stretch or shrink as a fraction of what the spaces allow): mean and total demerits (1 + badness)^2.
    mean_bad: f64,
    demerits: f64,
}

fn stats(doc: &wordcraft_doc::Document, l: &wordcraft_layout::DocLayout) -> Stats {
    let (mut lines, mut j, mut sum, mut worst, mut loose, mut hyphens) = (0usize, 0usize, 0.0f64, 0.0f64, 0usize, 0usize);
    let (mut bad_sum, mut demerits) = (0.0f64, 0.0f64);
    for page in &l.pages {
        for it in &page.items {
            let Placed::Lines { para, l0, l1, .. } = it else { continue };
            for li in *l0..*l1 {
                let line = &para.lines[li];
                lines += 1;
                if line.hyphen.is_some() {
                    hyphens += 1;
                }
                if li + 1 == para.lines.len() {
                    continue; // the last line of a paragraph is not justified
                }
                // Natural and actual width of the line's inner spaces.
                let (mut natural, mut actual, mut n) = (0.0f64, 0.0f64, 0usize);
                for k in line.c0..line.c1 {
                    let Some(c) = para.clusters.get(k) else { continue };
                    if c.kind == wordcraft_layout::para::ClKind::Space && k + 1 < line.c1 {
                        natural += c.adv as f64;
                        let a = line.xs.get(k + 1 - line.c0).copied().unwrap_or(0.0) - line.xs.get(k - line.c0).copied().unwrap_or(0.0);
                        actual += a as f64;
                        n += 1;
                    }
                }
                if n > 0 && natural > 0.0 {
                    let dev = (actual / natural - 1.0).abs();
                    let r = if actual >= natural { dev / 0.5 } else { dev / 0.25 };
                    let bad = 100.0 * r.powi(3);
                    bad_sum += bad;
                    let pen = if line.hyphen.is_some() { 50.0 } else { 0.0 };
                    demerits += (1.0 + bad + pen).powi(2);
                    j += 1;
                    sum += dev;
                    worst = worst.max(dev);
                    if dev > 0.5 {
                        loose += 1;
                    }
                }
            }
        }
    }
    let _ = doc;
    Stats {
        lines,
        justified_lines: j,
        mean_dev: if j > 0 { sum / j as f64 } else { 0.0 },
        worst_dev: worst,
        loose,
        hyphens,
        mean_bad: if j > 0 { bad_sum / j as f64 } else { 0.0 },
        demerits,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut files = Vec::new();
    let (mut out, mut pages) = (std::path::PathBuf::from("."), 2usize);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                out = args[i + 1].clone().into();
                i += 1;
            }
            "--pages" => {
                pages = args[i + 1].parse().unwrap_or(2);
                i += 1;
            }
            f => files.push(f.to_string()),
        }
        i += 1;
    }
    std::fs::create_dir_all(&out).ok();
    for f in files {
        let md = std::fs::read_to_string(&f).unwrap_or_default().replace("\r\n", "\n");
        let body = match md.strip_prefix("---\n").and_then(|r| r.find("\n---\n").map(|e| r[e + 5..].to_owned())) {
            Some(b) => b,
            None => md,
        };
        let mut doc = wordcraft_formats::markdown::import(&body);
        doc.settings.compat_mode = 15;
        doc.settings.auto_hyphenation = true;
        let last = doc.body.len().saturating_sub(1);
        doc.format_paragraphs(&Pos::body(0, 0), &Pos::body(last, 0), &|p| p.align = Some(Align::Justify)).ok();
        let name = std::path::Path::new(&f).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        println!("SCENE|{name}|{} blocks, {} words", doc.body.len(), body.split_whitespace().count());
        for (label, optimal) in [("greedy", false), ("optimal", true)] {
            let mut cache = LayoutCache::new();
            let opts = LayoutOptions { optimal_breaks: optimal, ..Default::default() };
            let t = Instant::now();
            let l = layout(&doc, &mut cache, &opts);
            let ms = t.elapsed().as_secs_f64() * 1000.0;
            let s = stats(&doc, &l);
            println!(
                "STATS|{name}|{label:7}|{} pages, {} lines, {} justified; space deviation mean {:.3} worst {:.2}; lines with spaces off by more than 50%: {}; hyphenated lines {}; mean badness {:.1}, total demerits incl. hyphen penalty {:.0}; layout {ms:.1} ms",
                l.pages.len(),
                s.lines,
                s.justified_lines,
                s.mean_dev,
                s.worst_dev,
                s.loose,
                s.hyphens,
                s.mean_bad,
                s.demerits
            );
            for (pn, page) in l.pages.iter().enumerate().take(pages) {
                let img = render_page(&doc, page, 1.5, &RenderOptions::default());
                if let Some(buf) = image::RgbaImage::from_raw(img.width, img.height, img.pixels) {
                    let path = out.join(format!("{name}-{label}-p{}.png", pn + 1));
                    buf.save(&path).ok();
                }
            }
        }
    }
}
