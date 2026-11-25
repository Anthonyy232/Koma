# Working on Koma

A screenshot tool for Windows: PrtSc freezes the desktop, you drag an area, mark it up, and copy or save it. The interface is Exact contract files, with Rust for the pixels and the Windows host. It reaches people as a single `Koma.exe` on GitHub Releases. `scripts/setup.ps1` fetches the Exact checkout it builds against.

## Code

- Simple and direct. No speculative abstractions, options or extension points.
- Stay in scope; suggest extras instead of adding them.
- Use what the platform, framework, dependencies and this repo already provide before writing a helper.
- Validate only at real boundaries (user input, storage, network, other processes). No checks, fallbacks or try/catch for cases that can't happen.
- One source of truth for each value or rule.
- Delete dead code you find or create.
- Match the surrounding code.
- Fix root causes, and fix the same pattern everywhere it occurs.

## Comments

- Only where the reason isn't clear from the code: a quirk, a race, a non-obvious decision.
- Short fragments, not sentences. Never restate code, narrate, or label sections.
- Delete comments that no longer match.

## Tests

- Behaviour and the edges that can break. No trivial, duplicate, obvious or low-signal tests.
- A regression test fails without its fix.
- Deterministic: fake time and observable events, no sleeps.
- A failing test is a bug until shown otherwise.

## Koma

- Exact is not in this repo. `scripts/setup.ps1` puts it in `vendor/exact` at the commit in `patches/exact.commit` with `patches/exact-overlay.patch` applied. Change Exact only through that patch, and remake the patch when the commit moves.
- Read `vendor/exact/docs/contract-for-agents.md` and `agent-pitfalls.md` before writing contract code.
- Behaviour belongs in the `.contract` files. Rust does only what a contract can't, and `data/` has no platform code.
- The contract tests run headless: they can't see the overlay, a capture ending or the real clipboard. Check those on the real desktop with `tools/peek`.
- Koma pictures whatever is on screen. Run it over invented content when capturing for docs, never the owner's windows.
- `site/` is the project website. It is not part of the program.
- The app id is the key to its storage and login item: never change it.

## Done means

- Diff reviewed against this file.
- `scripts/check.ps1` passes with zero warnings.
- UI changes seen in the running app on the real desktop. Docs and metadata changes need no end-to-end run.
- README updated for anything users will notice; behaviour changes called out.

## Docs

- README is for users: what it does, how to get it, how to use it. No internals, no agent instructions.
- This file is for agents: only what can't be found quickly in the code.
- No meta docs in the repo (notes, audits, plans, reports).
- Generated images, the app icon among them, are built from the orphan `media` branch; its README has the steps.

## Git

- `type: summary` or `type(scope): summary`: lowercase, imperative, no full stop, about 70 characters at most. Types: `feat` `fix` `refactor` `perf` `test` `docs` `style` `build` `ci` `chore`.
- Body only to say why. `Fixes #123` closes an issue.
- One logical change per commit. Squash fix-ups before merging.
- Branch off `main`. A single commit fast-forwards; several merge with `merge: <summary>`.
- No AI attribution in commits, branches, PRs or releases.
- One maintainer: amending and force-pushing to tidy history is fine. Keep contributors as authors. Delete stale branches.

## Pull requests

- Title in commit format. Body: what changed, why, how it was checked. Short.

## Releases

- Bump commit `chore: release vX.Y.Z`; tag `vX.Y.Z`, which must match the version in `Cargo.toml` and `windows/koma.manifest`.
- Title `Koma X.Y.Z`.
- Notes: banner image first, then `### New`, `### Improved`, `### Fixed` (only those that apply). One line per change, written for users; no internals, dependency lists or test counts. End with the install link(s) and the full changelog link.
- Banner: built and hosted on the `media` branch; reference its raw URL.
- The tag makes CI build `Koma.exe` and attach it to that version's release, drafting one if there is none. Publishing a draft creates the tag. The README links to that file name: never rename it.

## Agents

- Pushing, amending and force-pushing are allowed. Ask before publishing a release or submitting to a store.
- Leave changes you didn't make alone. Clean up scratch files, processes and emulators.
- Say what was checked and how, and what wasn't.
- Short replies; one recommendation, not a list of options.
