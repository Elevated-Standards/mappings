---
type: "always_apply"
---

# Created: 2025-09-20

# CHANGELOG Update Policy

This rule mandates updating the repository root CHANGELOG.md for any meaningful change and standardizes its format and workflow. It follows Keep a Changelog and Semantic Versioning.

- Root placement: CHANGELOG.md remains at repository root (this is an explicit exception to the docs-only markdown rule).
- Style reference: https://keepachangelog.com/en/1.0.0/
- SemVer: MAJOR.MINOR.PATCH

## When a CHANGELOG entry is required
Add an entry when a change is any of the following:
- Feature additions or behavioral changes impacting users/ops
- Bug fixes, security patches, or dependency upgrades
- Database schema/migration changes
- Public API or route changes; auth/permission policy changes
- Configuration/env variable additions/removals/renames
- Build/packaging/runtime changes (Docker/Compose under /ops)
- Performance, compatibility, or Tor-network behavior changes

Entries are not required for: pure refactors with zero user-visible effect, comments/whitespace only edits, or internal test-only refactors. If in doubt, add an entry.

## Required sections and format
Each release uses these section headers (include only those that apply):
- Added
- Changed
- Deprecated
- Removed
- Fixed
- Security

Date format: YYYY-MM-DD.

Example release skeleton:

- Keep the "Unreleased" section at top.
- On release, move items from Unreleased to a dated version, then re-create an empty Unreleased section.

## PR/Commit workflow requirements
- If your PR requires a changelog, include an entry under the Unreleased section.
- Reference issues/PRs using #123 and short commit shas when helpful.
- If no entry is needed, state "No changelog entry required" in the PR description.
- For DB or ENV changes, explicitly note migration steps and env variable keys in the entry.
- For /ops changes, explicitly note any port, service, or environment file changes.

## Release cut procedure
- Rename "Unreleased" to the new version: [X.Y.Z] - YYYY-MM-DD
- Add a fresh "Unreleased" section at the top
- Update compare links at the bottom:
  - [Unreleased]: compare vX.Y.Z...HEAD
  - [X.Y.Z]: compare vPREV...vX.Y.Z

## Tor-network specific note
When changes impact Tor behavior (Socks proxying, onion endpoints, isolation, port exposure), include a brief operational note in the entry for operators.

## Quality checklist for the entry
- Categorized under the correct section(s)
- Concise but clear, user/ops facing language
- Includes any breaking-change callouts and migration notes
- Contains references (issue/PR) when available
- Uses correct date and maintains Unreleased at top

## Minimal example (Unreleased)

- Added: New API v2 endpoints for feature requests.
- Fixed: Wallet RPC timeout handling (default 5000ms) to prevent hangs.
- Security: Bumped monero-php to 1.2.3 to address CVE-XXXX-YYYY.

## Enforcement guidance (manual)
Before merging:
- If code paths affecting users/ops changed, ensure CHANGELOG.md Unreleased contains an entry.
- For ENV/OPS/DB changes, verify the entry includes explicit keys, ports, or migration identifiers.

## Notes
- Keep entries free of inline HTML; plain Markdown only.
- Use imperative mood (e.g., "Add", "Fix").
- Keep line length reasonable; prefer bullets over paragraphs.

