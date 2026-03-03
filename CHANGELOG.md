# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog and this project adheres to Semantic Versioning for tagged releases.

## [Unreleased]

### Changed
- Optimized CI verification flow for better cache reuse:
  - removed redundant devflow bootstrap compilation from CI verify stage
  - switched verify execution to a single `dwf check:pr` gate
  - added npm cache persistence in GitHub Actions container runs
- Added devflow prune parity commands via host dependency extension:
  - `dwf setup:prune-runs`
  - `dwf setup:prune-containers`
  - `dwf setup:prune-deep`

### Documentation
- Added published server image run instructions in user and root docs.
- Added cleanup/pruning workflow guidance and command references.
- Aligned MyST site branding text with current workspace version `0.1.0-alpha.0`.

## [0.1.0-alpha.0] - 2026-03-04

### Added
- Initial multi-surface workspace bootstrap across Rust and TypeScript packages.
- Phase-2 architecture, workflow, testing, and execution documentation baseline.
