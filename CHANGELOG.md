# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Claude Code knowledge skill at `.claude/skills/sdi/` — a router-style
  `SKILL.md` plus task-keyed sub-files (`cli.md`, `config.md`, `embedding.md`,
  `invariants.md`) so contributors and embedders using Claude get surgical SDI
  knowledge on demand instead of preloading `CLAUDE.md`.
