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

- Keep each patch in its own commit, with a `Patch:` line in the message that names its entry here.
- Every generic patch is offered upstream and removed from this list when upstream has it. A patch that stays here needs a reason.
- Never edit upstream files for convenience; if a change is not a patch listed here it does not belong on `plotworx`.
- Licence: WordCraft is `MIT OR Apache-2.0`; keep its notices. The ArtCraft name, wordmark and logos in `docs/brand/` are trademarks and are **not** open source: do not ship them in Plotworx products.

## Carried patches

| # | Change | Files | Why | Upstream |
|---|---|---|---|---|
| 1 | Android system font directories | `crates/fonts/src/fontdb.rs` | `system_font_dirs()` has no Android case, so system fallback fonts (notably the colour emoji font `/system/fonts/NotoColorEmoji.ttf`) are never found and emoji draw as empty boxes. Adds `/system/fonts`, `/system/product/fonts`, `/product/fonts` for `target_os = "android"`. | Not yet offered |

## Pending (known, not yet carried)

| Change | Why | State |
|---|---|---|
| Keep-with-next lookahead through the paragraph cache | In `crates/layout/src/lib.rs`, `next_first_line` lays out the following paragraph from scratch with `para::layout_para` on every pass instead of through the paragraph-layout cache. On a 111k-word manuscript with headings this was 68-75% of a keystroke (layout 29-31 ms; 9-10 ms with the fix). | Verified against revision `e05b7fe` (3-9x faster, identical page output on the documents tested). Upstream has since changed this function (it now computes list labels for the lookahead), so the old patch no longer applies; it must be re-derived and revalidated (page-hash equality plus the layout suite) against current code before it is carried. |
