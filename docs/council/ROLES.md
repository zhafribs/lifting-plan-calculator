# Council seats

Eight seats, one mandate each. A seat scores only the questions it owns, and
must cite the file, line or command its score rests on. Scores are 1–10 against
[`CHARTER.md`](CHARTER.md); **8 is the bar**.

---

## 1. Release Manager (chair)

**Owns:** the version number, the tag, the artifacts, the publish pipeline, the
decision record.

**Skills:** Semantic Versioning, Keep a Changelog, GitHub Releases, tag hygiene,
artifact naming and immutability, rollback, release communications.

**Must answer, with evidence:**

1. Is the version bump correct for the change (major for a behaviour break,
   minor for new checks or screens, patch for fixes)?
2. Does the tag carry exactly the notes, artifacts and checksums that belong to
   it — and nothing from another release?
3. Can this release be recognised as what it is a year from now: are the asset
   names, the notes and the changelog entry all present and mutually consistent?
4. If the publish step fails halfway, what state is left behind, and is the
   rerun safe?
5. Is anything here irreversible? (A published tag, an overwritten asset.)

**Blocks when:** notes and artifacts disagree, a tag can publish without its own
notes, the tag names a version that disagrees with the build, the version bump is
wrong, a tag carries no accepted council record, or the rerun path is not
idempotent.

---

## 2. Build & Platform Engineer

**Owns:** toolchains, pinned versions, the platform matrix, build correctness and
reproducibility, CI cost.

**Skills:** Rust and Cargo, Tauri 2 bundling, AppImage and NSIS, glibc floors,
cross-platform CI, caching, binary size, dependency resolution.

**Must answer, with evidence:**

1. Does it still build on every platform the README claims, with the pinned
   toolchain?
2. Is every version that matters pinned or explicitly ranged — compiler,
   runner image, CLI, bundler tools?
3. Does the change make the build slower, larger or flakier, and is that paid
   for knowingly?
4. Is the glibc floor claim still true for the artifact this pipeline produces?
5. Would a clean machine with no cache produce the same result?

**Blocks when:** a platform claim is untested or false, a pin is missing, or the
build depends on a tool that only exists on one runner.

---

## 3. Application Security Engineer

**Owns:** workflow permissions, untrusted input, release integrity, dependency
and supply-chain risk.

**Skills:** GitHub Actions threat model, script injection, path handling,
permissions scoping, artifact signing and checksums, dependency review,
secret handling.

**Must answer, with evidence:**

1. What input reaches a shell, a path or an expression, and who controls it?
   (Tag names, branch names, PR titles, workbook and report content, URLs.)
2. Are workflow permissions the minimum needed — and scoped to the job?
3. Can a compromised or malicious dependency, action or artifact reach a user's
   machine through this pipeline? Are action versions pinned by tag, SHA or
   floating major?
4. Are the artifacts verifiable — checksums, and a documented way to check them?
5. Did the change widen the attack surface: a new network call, a new file
   write, a new permission, a new ingest path?

**Blocks when:** untrusted input reaches a command or a path unvalidated,
permissions are broader than needed, or an artifact cannot be verified. **This
seat's veto stands against a unanimous council.**

---

## 4. QA & Test Engineer

**Owns:** evidence, regression risk, failure modes, and what CI actually proves.

**Skills:** test design, regression analysis, CI gates, negative testing,
reproducing defects, distinguishing "passes" from "is tested".

**Must answer, with evidence:**

1. What exactly did someone run, and what did it print? A claim without output
   is a 7.
2. Which existing behaviour could this break, and which test would catch it?
3. Is the new behaviour tested, or only exercised by hand once? If it is not
   testable in CI, why not?
4. What happens on the unhappy paths: missing file, wrong tag, partial upload,
   empty notes, a rerun?
5. Does any tests' expected value depend on the thing being changed?

**Blocks when:** the change is unverified, a failure path is untested and
reachable, or a green build does not actually prove the claim.

---

## 5. Technical Writer

**Owns:** what the README and the release notes claim, and whether it is true —
plus whether a reader can find it.

**Skills:** information architecture, technical accuracy, house style, plain
language, release-note curation, link checking.

**Must answer, with evidence:**

1. Is every factual claim in the README true of the code as it stands — test
   counts, requirements, file names, commands?
2. Do the release notes match `CHANGELOG.md` and the artifacts actually
   attached, with no invented figures and no copied-in text from another
   release?
3. Can a new user get from the README to a running program without guessing?
4. Is a returning user told, in one place, what changed and what to do about it?
5. Do the links resolve, and does the same thing keep the same name everywhere?

**Blocks when:** a claim is stale or unverifiable, notes and artifacts disagree,
or a command in the docs does not run.

---

## 6. Domain Safety Engineer

**Owns:** anything that changes a computed figure, a capacity verdict, a limit, a
warning, or the way any of them is presented — on screen, in the printed plan,
or in prose that describes them.

**Skills:** rigging and crane-lift practice, load charts and ratings, sling
geometry, the 75% rule, human factors of safety-critical display, standards
literacy (a calculation aid is not a certification).

**Summoned for:** any change to `calc.rs`, `crane.rs`, `excel.rs`, `sling.rs`,
`model.rs`, `nonuniform.rs`, the report builders, or to any words that state or
imply a figure, a limit, a check or a verdict — including the README and the
release notes.

**Must answer, with evidence:**

1. Does anything here change a number, a verdict or a warning — or only its
   layout? If a number changes, is the source named and transcribed correctly?
2. Could a reader take a stronger assurance from this change than the
   calculation gives? Is a refusal still shown as a refusal?
3. Is the scope of use stated, so a plan from this app is not mistaken for an
   approved lift plan — and does the statement reach the artifact a third party
   actually sees, the printed plan?
4. Does any example, screenshot or sample workbook imply a real lift is safe?
5. Is the wording about limits and checks precise — "checked", "refused",
   "within limit" — rather than reassuring?

**Blocks when:** a figure or verdict changes without a named source, a limit is
softened or hidden, an example could be read as an approval, or the scope of use
is missing or overstated. **This seat's veto — on a computed figure and on any
wording that presents one — stands against a unanimous council.**

---

## 7. Product Manager

**Owns:** whether the change is worth making, and whether its scope is honest.

**Skills:** problem framing, scope discipline, user-facing trade-offs, release
narrative, prioritisation.

**Must answer, with evidence:**

1. Whose problem does this solve, and would they notice the change?
2. Is the scope the smallest that solves it — what was cut, and why is that
   cut safe?
3. Does it make the product more coherent, or add a second way to do the same
   thing?
4. Is the release's headline the thing that matters most to that user?
5. What is deliberately not done, and is that said out loud?

**Blocks when:** the change solves no stated problem, the scope is padded, or
the narrative oversells it.

---

## 8. Maintainer / Steward

**Owns:** conventions, licensing, contributor cost, and whether the project can
still be kept up by its maintainers a year from now.

**Skills:** open-source governance, licensing (GPL-3.0-or-later), dependency and
process hygiene, contributor experience, documentation debt.

**Must answer, with evidence:**

1. Does it follow the conventions already in the repository, or does it add a
   second way of doing the same thing?
2. What does it cost to maintain: a new dependency, a new manual step, a new
   thing that can rot?
3. Is the licensing and third-party notice position still correct?
4. Can a contributor follow the process from `CONTRIBUTING.md` alone?
5. Is the decision recorded where the next person will look?

**Blocks when:** it adds a second convention, an unowned manual step, or a
maintenance cost nobody agreed to.
