use wordcraft_doc::{Pos, props::Align};
use wordcraft_layout::{LayoutCache, LayoutOptions, Placed, layout};

fn para_demerits(l: &wordcraft_layout::DocLayout) -> Vec<(f64, Vec<(usize, f64, bool)>)> {
    // Per paragraph piece: total demerits and (line start, deviation, hyphen) per non-last line.
    let mut out = Vec::new();
    for page in &l.pages {
        for it in &page.items {
            let Placed::Lines { para, l0, l1, .. } = it else { continue };
            let mut tot = 0.0;
            let mut ls = Vec::new();
            for li in *l0..*l1 {
                let line = &para.lines[li];
                if li + 1 == para.lines.len() {
                    continue;
                }
                let (mut natural, mut actual, mut n) = (0.0f64, 0.0f64, 0usize);
                for k in line.c0..line.c1 {
                    let Some(c) = para.clusters.get(k) else { continue };
                    if c.kind == wordcraft_layout::para::ClKind::Space && k + 1 < line.c1 {
                        natural += c.adv as f64;
                        actual += (line.xs.get(k + 1 - line.c0).copied().unwrap_or(0.0) - line.xs.get(k - line.c0).copied().unwrap_or(0.0)) as f64;
                        n += 1;
                    }
                }
                if n > 0 && natural > 0.0 {
                    let dev = actual / natural - 1.0;
                    let r = if dev >= 0.0 { dev / 0.5 } else { -dev / 0.25 };
                    let pen = if line.hyphen.is_some() { 50.0 } else { 0.0 };
                    tot += (1.0 + 100.0 * r.powi(3) + pen).powi(2);
                    ls.push((line.start, dev, line.hyphen.is_some()));
                }
            }
            out.push((tot, ls));
        }
    }
    out
}

fn main() {
    let f = std::env::args().nth(1).unwrap();
    let md = std::fs::read_to_string(&f).unwrap().replace("\r\n", "\n");
    let body = match md.strip_prefix("---\n").and_then(|r| r.find("\n---\n").map(|e| r[e + 5..].to_owned())) {
        Some(b) => b,
        None => md,
    };
    let mut doc = wordcraft_formats::markdown::import(&body);
    doc.settings.compat_mode = 15;
    doc.settings.auto_hyphenation = true;
    let last = doc.body.len().saturating_sub(1);
    doc.format_paragraphs(&Pos::body(0, 0), &Pos::body(last, 0), &|p| p.align = Some(Align::Justify)).ok();
    let g = para_demerits(&layout(&doc, &mut LayoutCache::new(), &LayoutOptions::default()));
    let o = para_demerits(&layout(&doc, &mut LayoutCache::new(), &LayoutOptions { optimal_breaks: true, ..Default::default() }));
    for (i, (a, b)) in g.iter().zip(&o).enumerate() {
        if b.0 > a.0 + 1.0 {
            println!("PARA {i}: greedy {:.0} optimal {:.0}", a.0, b.0);
            println!("  greedy : {:?}", a.1.iter().map(|(s, d, h)| format!("{s}:{d:+.2}{}", if *h { "-" } else { "" })).collect::<Vec<_>>());
            println!("  optimal: {:?}", b.1.iter().map(|(s, d, h)| format!("{s}:{d:+.2}{}", if *h { "-" } else { "" })).collect::<Vec<_>>());
        }
    }
}
