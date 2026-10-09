# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.0] - 2026-10-09

The first public release: a Linux desktop build of the Lifting Plan Calculator,
written in Rust with a Tauri 2 shell and delivered as a portable AppImage.

### Added

- **Overall Weight** form: load lines and tackle, summed to the gross load at
  the hook.
- **Crane** form: capacity check against a load chart imported from Excel, or a
  single point typed by hand under the 75% rule.
- **Uniform Load** form: the nineteen 1-leg / 2-leg / nested / bridle
  arrangements, each checked against the gross load.
- **Nonuniform Load** form: the 2-leg asymmetric bridle with a shortening grab
  hook.
- **Tandem** form: two cranes sharing one load, in both declared lug
  configurations.
- **Summary** tab: an HTML report with KaTeX-typeset formulas that can be saved
  as a portable file or printed.
- Rust calculation engine ported from the Android Kotlin source, with 134 tests
  ported from the Kotlin unit suite.
- Two AppImage build paths: a fast host build and a portable Ubuntu 22.04
  userspace build (glibc 2.35 floor) that runs on Ubuntu 22.04+, Fedora 36+ and
  Arch.

### Fixed

- **Summary tab scrolling**: on narrow (stacked) windows the cards pinned over
  the report and the page had two scrollbars. The cards now leave the sticky
  layer below the breakpoint and the report is sized to its content, so the
  page owns a single scrollbar and the whole plan scrolls as one. The wide
  side-by-side layout keeps its independent report scroll.
