# Plotworx patches to WordCraft

This is the Plotworx fork of [storytold/wordcraft](https://github.com/storytold/wordcraft). WordCraft changes daily, so the fork follows upstream
continuously instead of freezing a version. These rules keep that cheap.

## Branches

| Branch | Purpose |
|---|---|
| `main` | An exact mirror of `upstream/main`. Fast-forward only. **Never commit to it.** |
| `plotworx` | `main` plus the patches listed below, kept as few and as small as possible. Plotworx depends on this branch. |

## Sync procedure (run on a regular schedule, not ad hoc)

1. `git fetch upstream && git checkout main && git merge --ff-only upstream/main && git push origin main`
2. `git checkout plotworx && git rebase main` (a conflict means a patch needs re-deriving; see "Pending" below).
3. Build and run the Plotworx gate against `plotworx` (latency matrix, input-correctness run, render checks, Android input checks) before anyone depends on the new revision.
4. Record the sync: upstream revision, patches carried, gate results (in the Plotworx runtime repository's `.work`).

A Plotworx build always records the exact revision it was built from (so a release can be reproduced); the policy is to keep moving that revision forward, not to freeze it.

## Rules for patches

- **Before adding or keeping a patch, search upstream issues and pull requests** and record the numbers with links in the register. Do not duplicate an open pull request: carry it, naming the PR, and drop it when upstream merges it.
- **Nothing is sent upstream without the owner's review of the exact diff and text** (pull requests, issues, comments).
- Keep each patch in its own commit, with a `Patch:` line in the message that names its entry here.
- Every generic patch is offered upstream and removed from this list when upstream has it. A patch that stays here needs a reason.
- Never edit upstream files for convenience; if a change is not a patch listed here it does not belong on `plotworx`.
- Licence: WordCraft is `MIT OR Apache-2.0`; keep its notices. The ArtCraft name, wordmark and logos in `docs/brand/` are trademarks and are **not** open source: do not ship them in Plotworx products.

## Carried patches

| # | Change | Files | Why | Upstream |
|---|---|---|---|---|
| 1 | Android system font directories | `crates/fonts/src/fontdb.rs` | `system_font_dirs()` has no Android case, so system fallback fonts (notably the colour emoji font `/system/fonts/NotoColorEmoji.ttf`) are never found and emoji draw as empty boxes. Adds `/system/fonts`, `/system/product/fonts`, `/product/fonts` for `target_os = "android"`. | **No upstream issue or PR.** Upstream's `docs/target-app-parity.md` says Android apps are out of scope; related feature requests: [#984](https://github.com/storytold/wordcraft/issues/984) (open), [#3499](https://github.com/storytold/wordcraft/issues/3499) (open, `priority: low`), [#1574](https://github.com/storytold/wordcraft/issues/1574) (closed as duplicate). Not submitted; owner decides whether to open an issue first. |
| 2 | **Carried upstream PR [#1292](https://github.com/storytold/wordcraft/pull/1292)** (tip `670f4425`; incremental pagination with the review follow-ups for issue [#1284](https://github.com/storytold/wordcraft/issues/1284); stacked on [#675](https://github.com/storytold/wordcraft/pull/675) / issue [#554](https://github.com/storytold/wordcraft/issues/554)) | `crates/layout`, `crates/engine`, `crates/doc` (merge commit `ad6a1357`) | Layout resumes from the edited paragraph instead of re-placing every block. 111k-word typing p95 about 4 ms against 16-18 ms; output equal to a full layout in every scenario tried (#675 alone is not, see #1284). **Replaces our earlier lookahead patch** (commit `90646cf5`, reverted in `a607d4e1`; tag `plotworx-patch2-lookahead-8e986366` keeps the commit our comment on PR [#2468](https://github.com/storytold/wordcraft/pull/2468) links to reachable). | Not ours: carried until merged upstream. **Track it:** `git fetch upstream pull/1292/head`; when its tip moves, re-merge and re-run the whole gate (equivalence and self-consistency checks, layout and engine tests, latency). Drop this entry when upstream `main` contains it. Owner decision 2026-10-22: carry it. |

## Validation of carried patches (re-run on every sync and whenever the carried PR's tip moves)

Carried PR #1292 at tip `670f4425` on upstream `8e986366`, evaluated 2026-10-22 (details in the Plotworx runtime's `.work/tasks/wordcraft-pr-evaluation-2026-10-22.md`):

- **Output equivalence:** the Plotworx `fp` tool hashes every placed page item of nine documents before and after eight edits: all nine identical to unpatched `main`. Its `--selfcheck` compares the live (incremental) layout with a from-scratch layout after every edit: **0 differences** across generated manuscripts (headings, numbered lists), WordCraft's own samples and the **page-break scenario from issue #1284** (80 edits). PR #675 alone fails that scenario (80 of 80 differ).
- **WordCraft's own tests:** layout 492 pass / 2 fail (two Thai-font tests that need fonts the test machine lacks; the same two fail on `main`, which has 487 / 2); engine 883 pass / 0 fail (`main`: 876 / 0).
- **Speed** (offscreen key-to-GPU-complete, RTX A5000, 111k-word document): typing median 3.7-3.9 ms, p95 4.0-4.7 ms (previously 13.4-15.9 ms and 15.7-18.4 ms); Enter with headings p95 about 10 ms.
- **Stale pages and structure (2026-10-22, Plotworx `fp --stale`):** strict comparison of the live layout with a from-scratch layout after each of 750 edits in five documents (books with tables, footnotes, floats, TOC, PAGE/NUMPAGES footer; samples), edits in body, table cells, footnotes, headers, footers and text boxes with section breaks, page breaks and text boxes inserted: **0 differences**. Positive control: #675 alone differs after 21 of 750. Long session (5,000 edits): memory levels off.
- **Known limit:** per-edit layout cost grows with page count in documents with tables, pictures and page-number footers (200-page synthetic book 14 ms median; upstream `main` 19 ms); logged as Plotworx runtime D-026.
- **Not covered:** equations, columns; Enter and Backspace-join are not incremental; the Read view.

Patch 1 (Android fonts): compiles on the host; not yet verified on the tablet through the patched crate.

## Upstream candidates (evaluated 2026-10-22, not carried; details in the Plotworx runtime `.work/tasks/wordcraft-pr-evaluation-2026-10-22.md`)

| Upstream | What | Result against the Plotworx gate |
|---|---|---|
| PR [#1292](https://github.com/storytold/wordcraft/pull/1292) (tip `670f4425`) for issue [#1284](https://github.com/storytold/wordcraft/issues/1284), stacked on #675 | Incremental pagination with the review fixes (page-break flag, part blocks, cache eviction) | **Carried** (owner decision 2026-10-22; see patch 2); replaced our earlier lookahead patch. 111k-word typing p95 about 4 ms against 16-18 ms; output equal to a full layout in every scenario tried (0 of 80 edits differed in the page-break scenario); layout tests 492 pass / 2 fail (same two Thai-font tests) vs 487 / 2; engine tests 883 / 0 vs 876 / 0. Untested: stale header, footnote and text-box pages, table cells, section breaks, cache growth. |
| PR [#675](https://github.com/storytold/wordcraft/pull/675) (tip `cc12c5a3`) / issue [#554](https://github.com/storytold/wordcraft/issues/554) | Incremental pagination for large documents | **Do not carry alone:** reproduces issue #1284 (80 of 80 page-break edits differ from a full layout). Same speed-up as #1292. |
| PR [#2635](https://github.com/storytold/wordcraft/pull/2635) (tip `c97f8f21`) / issue [#1109](https://github.com/storytold/wordcraft/issues/1109) | Redraw only the changed band of a page | App-canvas code (`ui-egui`); nothing to carry. Technique ported into Plotworx's own surface. |

## Pending (known, not yet carried)

| # | Change | Files | Why | Status |
|---|---|---|---|---|
| (candidate 3) | **Optimal (Knuth-Plass) line breaking for composition**, off by default (`LayoutOptions::optimal_breaks`) | `crates/layout/src/optimal.rs` (new), `para.rs`, `lib.rs`, `table.rs`; `crates/engine/src/session.rs` (one field); tests `tests_optimal.rs`; examples `layout/examples/optimal_bench.rs`, `engine/examples/optimal_compare.rs` | Owner requirement (2026-10-22): Desk's composition output uses Knuth-Plass; WordCraft's breaker is greedy. | **Spike on branch `feature/optimal-line-breaks`, not carried on `plotworx`.** Layout tests 506 pass / 2 fail (the same two Thai-font tests as before), engine 883 / 0. Upstream: no pull request exists to duplicate (searched for line breaking / Knuth in issues and PRs: see the Plotworx runtime note); nothing is sent upstream without the owner's review. |
