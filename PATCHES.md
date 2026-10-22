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
| 2 | Keep-with-next lookahead through the paragraph cache | `crates/layout/src/lib.rs` (`next_first_line`) | The lookahead for a kept paragraph (every heading) laid the next paragraph out from scratch with `para::layout_para` on every pass, bypassing the paragraph-layout cache. On a 111k-word manuscript with headings that was most of a keystroke. A paragraph with **no inline objects** now goes through `Ctx::para_labelled` (with the list label the lookahead already computes); a paragraph with inline objects (equations, inline text boxes, pictures) keeps the original path, because `para_key` advances the equation counter and the cached environment fits inline boxes, which the plain lookahead environment does not. | **Duplicates open upstream PR [#2468](https://github.com/storytold/wordcraft/pull/2468)** ("cache the keep-with-next lookahead shape through the paragraph cache", opened 2026-10-16, likely conflicting since PR [#2431](https://github.com/storytold/wordcraft/pull/2431) changed `next_first_line` on 2026-10-22) and overlaps open PR [#675](https://github.com/storytold/wordcraft/pull/675) (incremental pagination, issue [#554](https://github.com/storytold/wordcraft/issues/554)). **Do not submit.** Carry until #2468 or #675 merges, then drop. Evidence below. |

## Validation of carried patches (re-run on every sync)

Patch 2, re-derived 2026-10-22 against upstream `8e986366` (the first version was validated against `e05b7fe`):

- **Output equivalence:** the Plotworx `fp` tool lays out nine documents (100-scene manuscript with headings, plain, headings plus bullet lists, headings plus numbered lists, headings plus nested lists, and WordCraft's own handbook, letter, resume and report samples) and hashes every placed page item, before and after a fixed series of eight edits. **All nine fingerprints are identical** with and without the patch.
- **WordCraft's own layout tests:** 487 pass and 2 fail, both with and without the patch; the 2 failures are Thai-font tests that need fonts this machine lacks.
- **Speed** (offscreen key-to-GPU-complete, RTX A5000, 300 keystrokes in the middle of a 111k-word document): layout 33.5 ms to 12.9 ms, whole keystroke median 38.3 ms to 16.0 ms (headings); no change without headings or at 10k words.
- **Not covered:** documents with inline objects take the original path and are therefore unchanged; footnotes, floats, columns, tables after headings, other languages and the Read view were not exercised.

## Upstream candidates to evaluate (not carried)

| Upstream | What | Why we care |
|---|---|---|
| PR [#675](https://github.com/storytold/wordcraft/pull/675) / issue [#554](https://github.com/storytold/wordcraft/issues/554) | Incremental pagination for large documents (open, mergeable) | Whole-manuscript editing latency. Issue [#1284](https://github.com/storytold/wordcraft/issues/1284) records that it can differ from a full layout after a page break: run the layout-equivalence check against it. |
| PR [#2635](https://github.com/storytold/wordcraft/pull/2635) / issue [#1109](https://github.com/storytold/wordcraft/issues/1109) | Redraw only the changed band of a page on an edit (open) | The tablet's page re-raster cost. |

## Pending (known, not yet carried)

None at present.
