# Engineering Council

Every change to this repository — a feature, a fix, a wording change, a new
release — passes the council before it lands. The council is not a rubber stamp:
a change that scores below the bar does not proceed, and the objections become
the work.

This file is the charter. The seats and their individual rubrics are in
[`ROLES.md`](ROLES.md). Decisions are recorded in [`decisions/`](decisions/).

## The bar

A change proceeds only when **every participating seat scores it 8.0 or more out
of 10**, and the council's mean is at least 8.0.

- A seat scoring **below 8 blocks the change**. Its findings are required
  changes, not suggestions, and that seat re-scores after they are made.
- A seat that finds a **safety, security or data-integrity defect** states it as
  a blocker regardless of its total score.
- Nothing is "waived for now". If the fix is out of scope, the scope is cut.
- **8 means no material defect and evidence on file.** Not "looks fine".

| Score | Meaning |
| --- | --- |
| 10 | Exemplary: better than the repository's own bar; another project could copy it. |
| 9 | Ship quality with nothing worth changing. |
| 8 | Meets the bar. No material defect; every claim evidenced; the seat's own checklist is satisfied. |
| 7 | One material gap, or a claim without evidence. **Blocks.** |
| 5–6 | Several gaps, or the change is unverified in the seat's area. **Blocks.** |
| 1–4 | Wrong, unsafe, or breaks something. **Blocks.** |

A seat that cannot find a defect still has to say what it checked. `8` is a
judgement about the change, not politeness about the author.

## The seats

| Seat | Owns | Can block |
| --- | --- | --- |
| **Release Manager** (chair) | Versions, tags, artifacts, the publish pipeline, the record | Yes — as chair, on process and on the record |
| **Build & Platform Engineer** | Toolchains, pinned versions, the platform matrix, build reproducibility | Yes |
| **Application Security Engineer** | Workflow permissions, untrusted input, release integrity, dependencies | Yes, always |
| **QA & Test Engineer** | Evidence, regression risk, failure modes, what CI actually proves | Yes |
| **Technical Writer** | What the docs and release notes claim, and whether it is true | Yes |
| **Domain Safety Engineer** | Anything that changes a figure, a verdict, a limit, or the words describing one | Yes, always |
| **Product Manager** | Whether the change is worth making, and whether the scope is honest | Yes |
| **Maintainer / Steward** | Conventions, licensing, contributor cost, long-term upkeep | Yes |

The **Release Manager seats as chair** and records who sat. The chair summons
only the seats the change touches, and must summon:

- the **Domain Safety Engineer** and the **QA Engineer** for any change to
  `calc.rs`, `crane.rs`, `excel.rs`, `sling.rs`, `model.rs`, `nonuniform.rs`,
  the report builders, or to prose that states or implies a figure, a limit, a
  check or a verdict;
- the **Application Security Engineer** for anything under `.github/`, any
  dependency change, or anything that adds a network call, a file write or a
  permission;
- the **Technical Writer** for any user-facing text.

A documentation-only change usually needs the Technical Writer, the Product
Manager and the Maintainer.

## How a review runs

1. **Proposal.** One paragraph: what changes, for whom, and why now. The diff
   and the evidence are attached, not described.
2. **Summoning.** The chair names the participating seats. The implementer does
   not score their own work (conflict of interest); they answer questions only.
3. **Independent scores.** Each seat scores against its own rubric, citing the
   file, line or command the score rests on. No seat sees another's score
   before writing its own.
4. **Objections.** Blocking findings are listed as required changes, each one
   actionable and testable.
5. **Re-score.** The blocking seats re-score after the changes. A seat that
   still scores below 8 keeps blocking.
6. **Record.** An accepted record is written to [`decisions/`](decisions/),
   numbered `NNNN-<slug>.md` like the ADRs, and never rewritten — a later
   reversal is a new record that supersedes it.
7. **Land.** Merge, tag or publish only against an accepted record.

## Keeping the gate operable

The council is one maintainer. That does not lower the bar; it changes who
performs the passes:

- The maintainer may run each summoned seat as a **separate, independent review
  pass** — a different reader, a fresh context, a written rubric — rather than a
  meeting. The pass is recorded with the same evidence a second reviewer would
  give.
- The **implementer never scores their own work.** If the maintainer wrote the
  change, the scoring pass is done by someone else (a contributor, a reviewer,
  or an independent agent given the rubric and the diff). If no independent pass
  is available, the change does not land.
- A change that touches a figure, a verdict or a limit may not be accepted on a
  single pass by the implementer, ever.
- A trivial change — a comment, a broken link, a typo that changes no meaning —
  is reviewed by the Maintainer seat alone; the pull request says so, and no
  numbered record is written.

## The record and the ADR

Two stores, two jobs:

- [`docs/adr/`](../adr/) records **why the product behaves as it does** — the
  75% band, the refusal of an out-of-chart radius, a report layout. It is read
  when someone asks why the calculator does this.
- [`decisions/`](decisions/) records **the review of a change** — who sat, what
  they scored, what blocked, what was accepted and what was ruled out. It is
  read when someone asks why this was allowed to ship.

A council record links the ADRs it relies on; an ADR need not mention the
council. A change that alters a recorded product decision adds or supersedes the
ADR **and** is reviewed by the council.

## Deadlock

The chair breaks a tie on process, not on substance. The Domain Safety
Engineer's veto — on a change to computed figures, to a limit or verdict, or to
any wording that presents one — and the Application Security Engineer's veto on
release integrity stand even against a unanimous council. If the two conflict,
the change is cut until both can score it 8 or more.

## Scope

The council governs this repository's own changes. It does not certify the
calculator for site use; see *Scope of use* in the [README](../../README.md).
