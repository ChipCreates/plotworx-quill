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

When `plotworx` carries an upstream PR as merge commits, `git rebase main` replays the PR's own
history onto the new base. Since 2026-10-24 the documented sync instead merges `main` into a
`sync/<date>` branch from `plotworx`, so published history is preserved and the push is a
fast-forward; no force-push. A rebase is only used in an isolated, unpublished candidate with a
recovery ref.

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
| 1 | Android system font directories | `crates/fonts/src/fontdb.rs` | `system_font_dirs()` has no Android case, so system fallback fonts (notably the colour emoji font `/system/fonts/NotoColorEmoji.ttf`) are never found and emoji draw as empty boxes. Adds `/system/fonts`, `/system/product/fonts`, `/product/fonts` for `target_os = "android"`. | **No upstream issue or PR for the font directories.** Upstream's `docs/target-app-parity.md` says Android apps are out of scope; related feature requests: [#984](https://github.com/storytold/wordcraft/issues/984) (open), [#3499](https://github.com/storytold/wordcraft/issues/3499) (open, `priority: low`), [#1574](https://github.com/storytold/wordcraft/issues/1574) (closed as duplicate). Rechecked 2026-10-26: open PR [#4135](https://github.com/storytold/wordcraft/pull/4135) (Android app shell) does not touch `crates/fonts`; `system_font_dirs()` still has no Android case on `main` `2c99a427`. Not submitted; owner decides whether to open an issue first. |
| 2 | **Carried upstream PR [#1292](https://github.com/storytold/wordcraft/pull/1292)** (tip `8a610273` on 2026-10-26; incremental pagination with the review follow-ups for issue [#1284](https://github.com/storytold/wordcraft/issues/1284); stacked on [#675](https://github.com/storytold/wordcraft/pull/675) / issue [#554](https://github.com/storytold/wordcraft/issues/554)) | `crates/layout`, `crates/engine`, `crates/doc` (merge commit `ad6a1357`) | Layout resumes from the edited paragraph instead of re-placing every block. 111k-word typing p95 about 4 ms against 16-18 ms; output equal to a full layout in every scenario tried (#675 alone is not, see #1284). **Replaces our earlier lookahead patch** (commit `90646cf5`, reverted in `a607d4e1`; tag `plotworx-patch2-lookahead-8e986366` keeps the commit our comment on PR [#2468](https://github.com/storytold/wordcraft/pull/2468) links to reachable). | Not ours: carried until merged upstream. **Track it:** `git fetch upstream pull/1292/head`; when its tip moves, re-merge and re-run the whole gate (equivalence and self-consistency checks, layout and engine tests, latency). Drop this entry when upstream `main` contains it. Owner decision 2026-10-22: carry it. |
| 3 | **Optimal (Knuth-Plass) line breaking for composition**, off by default: `LayoutOptions::optimal_breaks` plans the breaks of simple paragraphs with a Knuth-Plass solver over the shaped clusters; `Session` preference `optimal_line_breaks` (default off) turns it on for export layouts | `crates/layout/src/optimal.rs` (new), `para.rs`, `lib.rs`, `table.rs`, `tests_optimal.rs`; `crates/engine/src/session.rs`, `tests.rs`; examples `layout/examples/optimal_bench.rs`, `engine/examples/optimal_compare.rs` | Owner requirement (2026-10-22): Plotworx Desk's composition output (final pagination, preview, PDF/DOCX) uses Knuth-Plass; WordCraft's breaker is greedy. Typing and the live page are never affected. | **No upstream issue or PR** (searched 2026-10-22: knuth, optimal line, line breaking, justification quality, paragraph composer; "total-fit" not searched, rate limit). Related, not duplicates: open PR [#3110](https://github.com/storytold/wordcraft/pull/3110) (Khmer / Southeast Asian line breaking and balanced justification) edits `para.rs` too, so expect a conflict at sync if both land; [#2642](https://github.com/storytold/wordcraft/pull/2642) (ICU4X Thai/Lao line breaking). Rechecked 2026-10-26 on `main` `2c99a427`: still no Knuth-Plass code or PR; #3110 and #2642 remain open and still edit `para.rs`. Not offered upstream; that needs the owner's approval of the exact diff and text. **Owner decision 2026-10-22: carry it.** |
| 4 | **English spelling supplement: 11 ordinary words** (`heard`, `bled`, `slid`, `goodnight`, `craftsmen`, `dockworker`, `woodsmoke`, `candlelit`, `destabilize`, `methodological`, `specificity`; inflections come from the rules) | `assets/spelling/en-extra.txt`, test in `crates/proof/src/lib.rs` | The Moby list lacks them; `heard` was flagged nine times in 16,000 words of fiction (Plotworx Desk D-040). Data only, in the file upstream already uses for this purpose. | **No upstream issue or PR** (searched 2026-10-23: dialogue, heard spelling, capitalize first word sentence, irregular verb dictionary). Rechecked 2026-10-26: none of the eleven words is in `main`'s `assets/spelling/en-extra.txt`, and no upstream issue/PR adds them. Generic: could be offered upstream as a data-only PR; not submitted, the owner approves the exact text first. **Owner decision 2026-10-23: carry it.** |
| 5 | **English grammar: a dialogue tag after `?"` or `!"` is not "Capitalize the first word of a sentence"** (`"How much have you written?" she asked.`) | `crates/proof/src/lib.rs` (`check_grammar`, sentence-start rule; test `a_dialogue_tag_after_a_question_or_exclamation_is_not_flagged`) | Ordinary in fiction; flagged with the suggestion `She` (Plotworx Desk D-041). A tag after `."`, a bare `?`/`!` and a real sentence start are still flagged; upstream's own tests for those pass unchanged. | **No upstream issue or PR** (searched 2026-10-23). Rechecked 2026-10-26: `main`'s `grammar_en` still sets `after_end` after `?"`/`!"`, so the false positive remains; the only related upstream item is German sentence-ending punctuation PR [#2664](https://github.com/storytold/wordcraft/pull/2664) (open, unrelated). Generic: could be offered upstream; not submitted, the owner approves the exact diff and text first. **Owner decision 2026-10-23: carry it.** |
| 6 | **Layout test build: import `Page` and `Rgb` in `tests_linear.rs`** (test-only, one line: `Page` in the `super::` list and `use wordcraft_doc::props::Rgb`) | `crates/layout/src/tests_linear.rs` | Upstream `main` `2c99a427` does not compile `wordcraft-layout`'s test target: commit `4bb3679e` replaced `use super::*` with an explicit list and omitted `Page` and `Rgb`. `gate / gate (ubuntu-24.04)` failed on that commit. Needed for the fork's required `cargo xtask ci` and layout tests to build at all. | **Carried upstream PRs [#4417](https://github.com/storytold/wordcraft/pull/4417) and [#4401](https://github.com/storytold/wordcraft/pull/4401)** (both open, same one-line fix). Drop this entry as soon as upstream merges either. |

## Validation of carried patches (re-run on every sync and whenever the carried PR's tip moves)

**2026-10-26 sync candidate `a4800fbb`** (upstream `main` `2c99a427803364354181d0bc20865f05880605f7`, workspace 0.11.0; carried PR #1292 head `8a610273c844c71700c312b71a1da02b95277995`; fork `plotworx` base `6fab82eb`): all six patches carried (1 Android font directories unchanged; 2 PR #1292 re-merged; 3 optimal line breaking re-based and adapted to the 0.11 `HfLinks` header/footer refactor; 4/5 unchanged; 6 new, test-only). Refreshed upstream search found no issue or PR that supersedes 1, 3, 4 or 5; PR #1292 remains open and unmerged. The fork's `cargo xtask ci` passed **all 8 stages** (fmt, clippy, 4,610 workspace tests with 0 failed and 5 skipped, i18n report, doctest, assets, layers, wasm) in a clean worktree with `CRAFT_FONTS_DIR` set to craft-fonts at the workflow's pin `b076417b5880bd4154bd22f00695b1d28c6bea83`. Without that input, three Thai substitution tests fail on a host that has a system Thai font (Plotworx runtime D-063); with it, all ten Thai tests pass. Runtime consumer gate against this candidate: `cargo xtask ci` all stages passed, 501 tests, 0 failed; input correctness 1,000 × 120 = 120,000 events, 0 mismatches, negative control 183/183; incremental/full-layout equivalence five documents × 150 edits = 750, 0 differences; band-render comparison 4/4 PASS (max 1/255, 0.0001-0.0006% differing). Android: the Android-specific patch lines are unchanged, but the surrounding font database changed. Host `cargo check -p wordcraft-fonts --target aarch64-linux-android` passes, and the attached onn 7 (Android 16/API 36) passed a device check with the harness at this revision: Gboard `commitText` typing, Enter and backspace, and the emoji palette inserting 😀 rendered in colour. With the harness's manual emoji-font add removed, `FontDb::global().has_family("Noto Color Emoji")` is true through the Android system font directories (patch 1) and 😀 still renders in colour. IME composing/preedit and clipboard were not re-run (runtime F-1 note).

**2026-10-24 sync candidate `6b161f5d`** (upstream `main` `cf2fe473`; carried PR #1292 head `3927aa5d`): input correctness 1,000 × 120 events, 0 mismatches, negative control 183/183; incremental/full-layout equivalence 750 edits, 0 differences; band-render comparison 4/4 PASS (maximum pixel difference 1/255); WordCraft engine 925 passed, integration 41 passed / 1 measurement ignored; WordCraft layout 518 passed / 2 missing-Thai-font failures. The same two layout failures reproduced on upstream `main` `cf2fe473` (499 passed / 2 failed); engine baseline was 917 passed. The runtime `cargo xtask ci` passed using local path overrides to this exact sync commit. Android-specific source lines did not change in this sync; no tablet/device check was run. The Android font-directory patch remains unverified on-device.

Latency on the 2026-10-26 candidate (offscreen key-to-GPU-complete, RTX A5000, 1440x900, 300 events + 30 warmup, ppp 1, load average 2.1-2.5):

| Case | Median ms | p95 ms |
|---|---:|---:|
| 10k words, typing | 1.29 | 1.91 |
| 111k words, typing | 3.31 | 3.79 |
| 111k words with headings, typing | 3.43 | 4.14 |
| 111k words with headings, Enter | 11.23 | 13.34 |
| 111k words with headings, paste | 12.08 | 13.25 |


Latency, offscreen key-to-GPU-complete on RTX A5000 (1440×900, 300 events + 30 warmup):

| Case | Median ms | p95 ms |
|---|---:|---:|
| 10k words, typing | 1.16 | 1.62 |
| 111k words, typing | 2.48 | 3.12 |
| 111k words with headings, typing | 2.60 | 3.24 |
| 111k words with headings, Enter | 9.54 | 11.35 |
| 111k words with headings, paste | 10.89 | 12.78 |

Carried PR #1292 at tip `670f4425` on upstream `8e986366`, evaluated 2026-10-22 (details in the Plotworx runtime's `.work/tasks/wordcraft-pr-evaluation-2026-10-22.md`):

- **Output equivalence:** the Plotworx `fp` tool hashes every placed page item of nine documents before and after eight edits: all nine identical to unpatched `main`. Its `--selfcheck` compares the live (incremental) layout with a from-scratch layout after every edit: **0 differences** across generated manuscripts (headings, numbered lists), WordCraft's own samples and the **page-break scenario from issue #1284** (80 edits). PR #675 alone fails that scenario (80 of 80 differ).
- **WordCraft's own tests:** layout 492 pass / 2 fail (two Thai-font tests that need fonts the test machine lacks; the same two fail on `main`, which has 487 / 2); engine 883 pass / 0 fail (`main`: 876 / 0).
- **Speed** (offscreen key-to-GPU-complete, RTX A5000, 111k-word document): typing median 3.7-3.9 ms, p95 4.0-4.7 ms (previously 13.4-15.9 ms and 15.7-18.4 ms); Enter with headings p95 about 10 ms.
- **Stale pages and structure (2026-10-22, Plotworx `fp --stale`):** strict comparison of the live layout with a from-scratch layout after each of 750 edits in five documents (books with tables, footnotes, floats, TOC, PAGE/NUMPAGES footer; samples), edits in body, table cells, footnotes, headers, footers and text boxes with section breaks, page breaks and text boxes inserted: **0 differences**. Positive control: #675 alone differs after 21 of 750. Long session (5,000 edits): memory levels off.
- **Known limit:** per-edit layout cost grows with page count in documents with tables, pictures and page-number footers (200-page synthetic book 14 ms median; upstream `main` 19 ms); logged as Plotworx runtime D-026.
- **Not covered:** equations, columns; Enter and Backspace-join are not incremental; the Read view.

- **Patch 3 (optimal line breaking), 2026-10-22:** layout tests 506 pass / 2 fail (the same two Thai-font tests as before the patch), engine tests 884 / 0; 15 new tests (solver, layout modes, cache separation, export preference); with the option off the layout is unchanged (the existing 500+ layout tests pass untouched). Quality on 18 real scenes: total Knuth-Plass demerits -28.6%, better or equal on 16; cold composition of 96,000 words +9% (143 against 132 ms); see the Plotworx runtime note `knuth-plass-composition-spike-2026-10-22`. **Not covered:** parity with Desk's own composition output; paragraphs with tabs, objects, floats, list labels, drop caps or bidi (greedy as before); non-English text.

Patch 1 (Android fonts): compiles on the host; not yet verified on the tablet through the patched crate.

## Upstream candidates (evaluated 2026-10-22, not carried; details in the Plotworx runtime `.work/tasks/wordcraft-pr-evaluation-2026-10-22.md`)

| Upstream | What | Result against the Plotworx gate |
|---|---|---|
| PR [#1292](https://github.com/storytold/wordcraft/pull/1292) (tip `670f4425` at evaluation time) for issue [#1284](https://github.com/storytold/wordcraft/issues/1284), stacked on #675 | Incremental pagination with the review fixes (page-break flag, part blocks, cache eviction) | **Carried** (owner decision 2026-10-22; see patch 2); replaced our earlier lookahead patch. 111k-word typing p95 about 4 ms against 16-18 ms; output equal to a full layout in every scenario tried (0 of 80 edits differed in the page-break scenario); layout tests 492 pass / 2 fail (same two Thai-font tests) vs 487 / 2; engine tests 883 / 0 vs 876 / 0. Untested: stale header, footnote and text-box pages, table cells, section breaks, cache growth. |
| PR [#675](https://github.com/storytold/wordcraft/pull/675) (tip `cc12c5a3`) / issue [#554](https://github.com/storytold/wordcraft/issues/554) | Incremental pagination for large documents | **Do not carry alone:** reproduces issue #1284 (80 of 80 page-break edits differ from a full layout). Same speed-up as #1292. |
| PR [#2635](https://github.com/storytold/wordcraft/pull/2635) (tip `c97f8f21`) / issue [#1109](https://github.com/storytold/wordcraft/issues/1109) | Redraw only the changed band of a page | App-canvas code (`ui-egui`); nothing to carry. Technique ported into Plotworx's own surface. |

## Pending (known, not yet carried)

| Candidate | Owner decision | Status |
|---|---|---|
| **Task-item checkbox** for Plotworx list items (`- [ ]` / `- [x]` round trip, click to toggle) | 2026-10-22: add a checkbox of our own, patching WordCraft only if needed | **Evaluated 2026-10-22 (E4.1-5); no fork patch needed.** The checkbox *content control* (`ControlKind::CheckBox`, `developer.checkBox`/`toggleCheckBox`) is not list-aware: Enter after it does not continue it, its state is a symbol between control markers, deleting the range leaves the markers, and Plotworx's own surface does not draw controls. The carrier is WordCraft's native bullet list whose numbering defines a check box glyph: `☐` open, `☑` done, with the registry naming the pair. The existing `para.bullets { "kind": "☐" }` / `{ "kind": "☑" }` command switches a selected paragraph between the two lists and is undoable, so toggling needs no new command. The scene codec maps `- [ ]`/`- [x]` to those two lists and back (`crates/plotworx-scene-codec/tests/task_items.rs`). Remaining shell rule (E4.2-3, not a patch): after Enter in a *checked* item, switch the new item to `☐`. |
