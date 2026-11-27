# Product decisions

**Status:** 27 November 2026. These are questions, not work. Nothing here is
implemented. Each one needs a decision before a plan; the recommendation is the
maintainer's, and a decision is recorded as an ADR plus a council record.

## 1. Saving a plan

**Today:** nothing is stored between launches. The privacy contract allows one
GitHub update check at launch and one file write (`~/.cache/lifting-plan-calculator/ui.log`).
Closing the app loses the plan.

**Why it matters:** a lift plan takes long enough to enter that losing it is
painful, and users may expect a recent plan to reopen. Any storage changes the
privacy promise, which is currently a selling point.

**Options**

| Option | What it changes | Cost |
| --- | --- | --- |
| **A. Explicit export/import a plan file** (user picks the path) | Nothing is stored unless the user asks; the contract stays true | A schema with a version, validation at the boundary, a save/open dialog, tests |
| **B. Auto-save the last plan** in the app data folder | Reopens where you left off | Breaks the "nothing stored" promise; needs retention and a delete control |
| **C. No saving** | — | Work is lost on close |

**Recommendation:** A, if users ask. It keeps the privacy contract literal —
nothing is written unless the operator chooses a file — and it doubles as a way
to hand a plan to a colleague. A schema needs a version field and a strict
reader, because the file is untrusted input.

**A decision needs:** an ADR, the Security seat for the file dialog and the
schema, the Safety seat for every figure the file carries, and a council record.

**Drafted:** `docs/adr/0010-plan-file.md` and
`docs/superpowers/specs/2026-11-28-plan-file.md` are ready for review; nothing is
implemented.

## 2. Several lifts in one report

**Today:** one lift per report. Tandem is the only multi-crane case. The report
can break pages between sections, and sections can be switched off.

**Why it matters:** a real job often has several lifts; printing one report per
lift is honest but repetitive.

**Options**

| Option | What it changes | Cost |
| --- | --- | --- |
| **A. One report per lift** | — | Repetition |
| **B. Several lifts in one document** | One print job, one cover, one sign-off | A plan model with N lifts; every figure, verdict and footer must stay attached to its lift; page breaks and section toggles must work per lift |
| **C. A job-level summary plus per-lift pages** | Adds a cover with the worst verdict | The summary must not imply an approval the app cannot give |

**Recommendation:** A until a user asks. B is feasible — the page-break
machinery is already there — but the risk is exactly where the app is
conservative: a reader must never mistake one lift's verdict for another's.

**A decision needs:** a spec, an ADR, and the Safety seat on the per-lift
figures and the summary wording.

## 3. Language and units

**Today:** English, metric. The chart is the only capacity check, its radii are
metric, and the 75% rule is unit-free.

**Why it matters:** Malaysian practice mixes metric and imperial references, and
some users read Malay more comfortably.

**Options**

| Option | What it changes | Cost |
| --- | --- | --- |
| **A. English, metric** | — | Nothing |
| **B. Locale strings (en/ms)** | Labels and report prose | A string table, a language setting, a translation review of every figure's wording |
| **C. Unit entry and display** (t/kg, m/ft) | Inputs and figures convert at the edge | Every figure needs a shown unit; the chart stays metric, so a converted radius must not be fed to it silently |

**Recommendation:** B only if the users ask; C is the risky one, because a
converted figure that reaches the chart or the report without its unit is a
safety defect. If C is wanted, it needs a design where the unit is part of the
value everywhere, and the report prints the unit beside every number.

**A decision needs:** an ADR, a wording review of every figure and verdict, and
the Safety seat.

## How a decision becomes work

1. Write the ADR (what changes, what stays, the trade-off).
2. Get the relevant seats (Safety for figures, Security for inputs and files,
   Design for wording and UI) to 8/10.
3. Write an accepted council record naming the release.
4. Land it with a release note that says what changed for the user.
