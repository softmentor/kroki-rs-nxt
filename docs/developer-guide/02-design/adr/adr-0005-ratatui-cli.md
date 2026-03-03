---
title: "ADR-005: Ratatui CLI TUI"
label: kroki-rs-nxt.adr.0005
---

# ADR-005: Ratatui for CLI Terminal User Interface

## Status

**Accepted**

## Context

kroki-rs uses a basic `clap` CLI with subcommands (`serve`, `convert`, `batch`). For kroki-rs-nxt, we want a richer terminal experience with interactive mode, live preview, and better UX for diagram workflows.

## Decision

Use **Ratatui** as the TUI framework for the CLI surface (`apps/cli`), supplemented by `clap` for argument parsing and non-interactive mode.

## Consequences

**Positive:**
- Interactive diagram selection and preview in terminal
- Richer feedback during batch operations (progress bars, status)
- Modern terminal UI patterns (panels, tabs, scrolling)

**Negative:**
- Additional dependency and learning curve
- Terminal compatibility considerations across platforms
- Non-interactive mode must still work cleanly for CI/scripting

**Mitigation:** Maintain `clap` subcommands for non-interactive usage. Ratatui mode is opt-in via `--interactive` or default when TTY is detected.
