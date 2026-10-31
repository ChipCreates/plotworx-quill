# Quill

Quill is the editor core of the Plotworx applications (Desk, the Android editor, and the shared editor surface in `plotworx-runtime`). It is its own component with its own name, history and change log. It began on 2026-10-22 as a fork of [WordCraft](https://github.com/storytold/wordcraft) by the ArtCraft team and was renamed from `plotworx-wordcraft` on 2026-10-31.

## What stays the same, on purpose

- **The code base and crate names.** The crates are still called `wordcraft-*` (`wordcraft-doc`, `wordcraft-layout`, ...). Renaming them would change every import in Plotworx and every upstream merge; it is not planned.
- **The upstream relationship.** This repository is still a GitHub fork of `storytold/wordcraft`, with an `upstream` remote. Quill is not required to follow upstream on a schedule, but it can always import upstream changes.
- **The licence.** Upstream's code is `MIT OR Apache-2.0` (see `LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`). Quill's own changes are offered under the same terms. Copyright of upstream material stays with the ArtCraft Team and the WordCraft contributors; keep `NOTICE` and `ATTRIBUTION.md`.

## Branches

| Branch | Purpose |
|---|---|
| `main` | An exact mirror of `upstream/main` (`storytold/wordcraft`). Fast-forward only. Never commit to it. Keeps GitHub's "Sync fork" working. |
| `plotworx` | **Quill's trunk**: `main` plus Quill's own changes (see [PATCHES.md](PATCHES.md)). Plotworx depends on this branch by git URL. |

## Importing upstream changes

On demand, when the owner or a task wants them (no daily or weekly obligation; workspace `AGENTS.md`, "Upstream dependencies: Quill"). The procedure and the gate are in [PATCHES.md](PATCHES.md#import-procedure). Keep Quill's changes mergeable: prefer new crates or modules to edits of upstream files, so imports stay cheap. Never send anything upstream (pull requests, issues, comments) without the owner's review of the exact diff and text.

## Names and marks

The ArtCraft name, wordmark and logos are trademarks of the ArtCraft Team and are not covered by the licence. Upstream's README says forks and modified versions must remove them, so Quill does not ship them; `docs/brand/` was removed on 2026-10-31. Quill carries no WordCraft or ArtCraft branding of its own. Credit stays in `NOTICE`, `ATTRIBUTION.md` and the links above.

Upstream's `README.md`, `AGENTS.md`, `CLAUDE.md` and `ROADMAP.md` are left exactly as upstream has them, so imports do not conflict; they describe WordCraft, not Quill. GitHub shows `.github/README.md` instead of the root `README.md`.
