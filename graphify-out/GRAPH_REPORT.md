# Graph Report - lifting-plan-calculator  (2026-11-16)

## Corpus Check
- 117 files · ~132,272 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 37 file(s) not represented in the graph (top: .woff2 22, (none) 6, .xml 2)

## Summary
- 1818 nodes · 4004 edges · 115 communities (88 shown, 27 thin omitted)
- Extraction: 91% EXTRACTED · 9% INFERRED · 0% AMBIGUOUS · INFERRED: 373 edges (avg confidence: 0.82)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `8af26fbb`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- el
- Vn
- Crane Engine & Tests
- Lifting Plan Calculator README
- Release Notes 2.1.x
- Graph Screen UI
- Sling Engine & Tests
- Report Builder & Load Ratio
- commands.rs
- KaTeX Minified Core
- Excel Cell Data
- CI Workflow
- Calc Engine Primitives
- CraneInputs
- read_chart
- Geometry Frontend
- pdf_print.rs
- Report Capacity Tests
- Report Uniform Tests
- report_nonuniform.rs
- dump_fixtures.rs
- Tauri Configuration
- solve_nonuniform
- matrix_chart
- KaTeX Font Metrics
- Bug Report Template
- 0009 — Taking the dependency refresh
- Model & Density
- Dependabot Config
- Release Notes Checker
- LoadType
- Feature Request Template
- Release Workflow Env
- with
- update.rs
- jobs
- Lifting Plan Calculator 2.8.2
- Frontend Store & Charts
- Engineering Council Charter
- pdf.rs
- tandem_sections
- Release Notes Process
- KaTeX Parser Internals
- Publish Permissions
- LoadMeasurements
- chart_table
- KaTeX HTML Nodes
- KaTeX DOM Attributes
- AppImage Build Scripts
- Issue Template Config
- Tauri Permissions
- super
- KaTeX Trusted Types
- KaTeX DOM Classes
- Contributing Guide
- KaTeX Markup Nodes
- Hr
- KaTeX Class Nodes
- KaTeX Style Nodes
- dto.rs
- CalcException
- htmlBuilder
- K
- KaTeX Markup Base
- ADR-0001 Rated Capacity Comes Only from a Charted Point
- Republish Script
- LoadChartData
- Repository Root
- 0002 — A human-friendly plan head
- 0003 — Printed-plan cards, PDF export, footer and working-range diagram
- 0005 — Print the plan with the app's own web view
- Lifting Plan Calculator 2.5.2
- Lifting Plan Calculator 2.5.3
- Lifting Plan Calculator 2.5.4
- Changelog
- UsageState
- Lifting Plan Calculator 2.6.0
- env
- Lifting Plan Calculator 2.8.0
- pdf-export
- 0012 — Releases are signed, and the app will not run an update it cannot attribute
- Lifting Plan Calculator 2.10.0
- Lifting Plan Calculator 2.8.1
- 0010 — The workbook cannot exhaust memory, and Windows opens files without a shell
- linux-appimage
- NonuniformResult
- 0011 — The update download is verified, and the build tools are pinned
- 0005 — The workbook names the crane, and the chart is the only check
- 0006 — A quiet pipeline: audited dependencies, no compiler warnings
- 0007 — The last two warnings in the release log
- Lifting Plan Calculator 2.9.0
- Lifting Plan Calculator 2.9.1
- Lifting Plan Calculator 2.9.2
- env
- ADR-0005 — The load chart is the only capacity check
- governing_case_words
- on
- max_by_first
- update-signing-key.sh
- createPdfViewer
- Lifting Plan Calculator 2.9.4
- sign-manifest
- LoadItem
- Lifting Plan Calculator 2.9.3
- 0006 — The report preview is the PDF
- Lifting Plan Calculator 2.7.0
- check-vendored.sh

## God Nodes (most connected - your core abstractions)
1. `_` - 58 edges
2. `el()` - 55 edges
3. `createGraphScreen()` - 52 edges
4. `solve_crane()` - 42 edges
5. `LoadChartData` - 36 edges
6. `solve_one_leg()` - 34 edges
7. `build_report_html()` - 34 edges
8. `fmt()` - 31 edges
9. `createReportScreen()` - 31 edges
10. `createCraneScreen()` - 28 edges

## Surprising Connections (you probably didn't know these)
- `75% Rule Capacity Band` --conceptually_related_to--> `Capacity Is a Rating Only at a Specified Radius`  [INFERRED]
  README.md → docs/adr/0001-rated-capacity-envelope.md
- `75% Rule Capacity Band` --references--> `Out-of-Chart Radius Refusal`  [INFERRED]
  README.md → docs/adr/0001-rated-capacity-envelope.md
- `Application Security Engineer Seat` --conceptually_related_to--> `Threat Surface: xlsx Reading, Report Write, Embedded Web View`  [INFERRED]
  docs/council/ROLES.md → SECURITY.md
- `Draft-First Publish with Immutable Assets` --conceptually_related_to--> `Release Cut Process`  [INFERRED]
  docs/council/decisions/0001-per-tag-release-notes.md → CONTRIBUTING.md
- `Pull Request Template` --conceptually_related_to--> `Contributing Guide`  [INFERRED]
  .github/PULL_REQUEST_TEMPLATE.md → CONTRIBUTING.md

## Import Cycles
- 2-file cycle: `src-tauri/src/report.rs -> src-tauri/src/report_capacity.rs -> src-tauri/src/report.rs`
- 2-file cycle: `src-tauri/src/report.rs -> src-tauri/src/report_uniform.rs -> src-tauri/src/report.rs`
- 2-file cycle: `src-tauri/src/report.rs -> src-tauri/src/report_nonuniform.rs -> src-tauri/src/report.rs`
- 2-file cycle: `src-tauri/src/calc.rs -> src-tauri/src/model.rs -> src-tauri/src/calc.rs`

## Hyperedges (group relationships)
- **Engineering Council Review Gate** — docs_council_charter_engineering_council, docs_council_charter_eight_of_ten_bar, docs_council_charter_solo_maintainer_gate, docs_council_charter_domain_safety_veto, docs_council_roles_council_seats, docs_council_decisions_0000_template_record_template, contributing_council_review [EXTRACTED 1.00]
- **Cross-platform distribution and update pipeline** — docs_releases_v220, docs_releases_v240, docs_releases_v220_windows_build, docs_releases_v220_appimage_delta_updates, docs_releases_v220_release_asset_naming, docs_releases_v240_update_reminders, docs_releases_v240_windows_install_action, docs_releases_v220_glibc_floor [EXTRACTED 1.00]
- **Crane load-chart correctness: workbook import, 75% rule, booked rating, refusal** — docs_releases_v230, docs_releases_v250, docs_releases_v251, docs_releases_v230_workbook_crane_position, docs_releases_v230_seventy_five_percent_rule, docs_releases_v250_booked_crane_rating, docs_releases_v250_radius_refusal, docs_releases_v250_unit_detection [EXTRACTED 1.00]
- **2.1.x wide-window layout rework (scroll panes, split cut, rail footer)** — docs_releases_v210, docs_releases_v211, docs_releases_v212, docs_releases_v213, docs_releases_v214, docs_releases_v215, docs_releases_v210_independent_scroll_panes, docs_releases_v214_two_column_layout, docs_releases_v210_rail_footer [EXTRACTED 1.00]
- **Per-Tag Release Notes Pipeline** — changelog_changelog, changelog_unreleased_section, contributing_release_process, docs_releases_readme_release_notes_process, docs_releases_readme_notes_file_shape, docs_council_decisions_0001_per_tag_release_notes_one_file_per_tag, docs_council_decisions_0001_per_tag_release_notes_release_gate, docs_council_decisions_0001_per_tag_release_notes_draft_first_publish [EXTRACTED 1.00]
- **Safety Verdict Integrity (Band, Refusal, Booked Check, Scope)** — readme_seventy_five_percent_rule, readme_scope_of_use, docs_adr_0001_rated_capacity_envelope_out_of_chart_refusal, docs_adr_0001_rated_capacity_envelope_charted_point_principle, docs_adr_0002_booked_rating_check_booked_check, docs_council_roles_domain_safety_engineer [INFERRED 0.85]

## Communities (115 total, 27 thin omitted)

### Community 0 - "el"
Cohesion: 0.05
Nodes (110): api, errorText(), main(), go(), runInstaller(), showReport(), showUpdate(), toast() (+102 more)

### Community 1 - "Vn"
Cohesion: 0.08
Nodes (6): Cr, Hn, Mn, Nr, Vn, zn

### Community 2 - "Crane Engine & Tests"
Cohesion: 0.07
Nodes (54): ChartAction, Boom, Forget, Jib, Manual, Radius, a_hand_typed_line_carries_all_four_of_its_columns_into_the_result(), a_jib_angle_is_implied_from_the_radius_for_the_lower_solution() (+46 more)

### Community 3 - "Lifting Plan Calculator README"
Cohesion: 0.21
Nodes (15): Contributor Covenant Code of Conduct 2.1, Community Impact Enforcement Ladder, Application Security Engineer Seat, Five Calculation Forms Over One Shared Lift, Release Notes v2.0.0, Excel Load Chart Import (.xlsx), KaTeX-typeset HTML Report, Lifting Plan Calculator README (+7 more)

### Community 4 - "Release Notes 2.1.x"
Cohesion: 0.06
Nodes (53): Lifting Plan Calculator 2.1.0 release notes, Independent scroll panes on wide windows, Rail footer pinned beneath the icon rail, Scope of use — calculation aid, not a certified lift plan, Always-visible working range diagram (Graph tab), Lifting Plan Calculator 2.1.1 release notes, Lifting Plan Calculator 2.1.2 release notes, Overall Weight tab (+45 more)

### Community 6 - "Graph Screen UI"
Cohesion: 0.12
Nodes (44): createGraphScreen(), apply(), applyWorkbook(), checkboxOption(), drawAngleArc(), drawAxesGround(), drawBoom(), drawCrane() (+36 more)

### Community 7 - "Sling Engine & Tests"
Cohesion: 0.11
Nodes (33): a_bridle_is_rated_on_one_leg_fewer_than_it_has(), a_missing_shape_measurement_is_named_rather_than_treated_as_zero(), a_one_leg_sling_is_never_derated_by_a_leftover_tag_angle(), a_rectangular_choke_flare_must_stay_inside_the_vertical(), a_sling_too_short_for_the_wrap_names_the_length_it_needs(), a_two_leg_sling_does_take_its_rated_angle(), AF_DECIMALS, an_arrangement_that_cannot_reach_the_hook_says_so() (+25 more)

### Community 8 - "Report Builder & Load Ratio"
Cohesion: 0.07
Nodes (53): build_report(), PlanSignoff, a_blank_crane_says_not_entered_rather_than_zero(), a_chart_with_no_capacity_omits_the_usage_equation(), a_chosen_margin_reaches_the_page_that_is_printed(), a_crane_check_that_could_not_be_made_is_named_in_the_plan(), a_null_figure_is_not_a_zero(), a_plan_with_no_details_still_prints_the_blank_lines() (+45 more)

### Community 9 - "commands.rs"
Cohesion: 0.07
Nodes (50): a_failed_export_leaves_the_chosen_file_alone(), a_finished_pdf_replaces_the_chosen_file(), a_newer_preview_supersedes_an_older_one(), a_report_key_changes_with_the_report_and_its_footer(), an_external_program_is_started_outside_the_appimage(), an_interrupted_save_is_cleared_but_a_fresh_one_is_left_alone(), app_info(), APPIMAGE_ENV (+42 more)

### Community 10 - "KaTeX Minified Core"
Cohesion: 0.09
Nodes (15): _, ct(), $e(), handler(), It(), je(), mt(), N() (+7 more)

### Community 11 - "Excel Cell Data"
Cohesion: 0.16
Nodes (27): CellValue, Number, Text, chart_number(), declared_unit(), detect_header_row(), detect_matrix(), detect_unit() (+19 more)

### Community 12 - "CI Workflow"
Cohesion: 0.05
Nodes (47): name, runs-on, steps, concurrency, cancel-in-progress, group, env, CARGO_TERM_COLOR (+39 more)

### Community 13 - "Calc Engine Primitives"
Cohesion: 0.18
Nodes (21): a_zero_quantity_line_still_carries_its_weight_on_a_load_line_but_not_on_tackle(), CalcResult, chart_usage_percent(), clamp_qty(), crane_usage_band(), CRANE_USAGE_RATIO, DEFAULT_DENSITY_KG_M3, design_weight() (+13 more)

### Community 14 - "CraneInputs"
Cohesion: 0.19
Nodes (15): chart_update(), ChartHolder, ChartUpdate, import_chart(), CraneInputs, ChartImport, ChartTable, table_for() (+7 more)

### Community 15 - "read_chart"
Cohesion: 0.07
Nodes (48): JibOption, a_boom_table_lists_only_its_own_charted_radii(), a_boom_with_no_charted_rows_blanks_the_mirror_instead_of_keeping_the_old_one(), a_chart_that_declares_nothing_still_falls_back_to_its_figures(), a_column_chart_takes_the_unit_from_its_title_when_the_header_is_silent(), a_column_workbook_parses_to_the_references_own_table(), a_crane_title_line_names_the_machine_and_its_rating(), a_declared_kg_chart_is_not_read_as_tonnes() (+40 more)

### Community 16 - "Geometry Frontend"
Cohesion: 0.15
Nodes (23): ANGLE_MAX, ANGLE_MIN, angleForTip(), angleForWorkingRadius(), assemblyTip(), boomTip(), clamp(), CONFIG_BOOM (+15 more)

### Community 17 - "pdf_print.rs"
Cohesion: 0.06
Nodes (14): copy_tree(), main(), begin_print(), begin_print_with(), finish(), Job, print_to_pdf(), start() (+6 more)

### Community 18 - "Report Capacity Tests"
Cohesion: 0.14
Nodes (27): a_nested_lift_names_both_slings_throughout(), a_single_leg_prints_no_tag_capacity_block(), CHOKE_KEYS, count_h2(), direct_hook(), every_arrangement_carries_the_tag_rating_rule_when_it_has_more_than_one_leg(), label_index(), nested3() (+19 more)

### Community 19 - "Report Uniform Tests"
Cohesion: 0.13
Nodes (29): card_grid(), a_nested_input_section_names_its_piles_and_its_second_sling(), a_two_leg_nested_system_has_no_pile_row(), configuration_label(), dimension_rows(), every_arrangement_produces_a_geometry_section_of_its_own(), geometry(), input_card() (+21 more)

### Community 20 - "report_nonuniform.rs"
Cohesion: 0.17
Nodes (25): TandemResult, installed_tag_block(), nonuniform_rating_legs_note(), heading(), label_or_not_entered(), label_value(), a_block_that_was_never_set_up_says_which_tab_to_complete(), a_crane_with_no_chart_capacity_says_so_rather_than_showing_zero_percent() (+17 more)

### Community 21 - "dump_fixtures.rs"
Cohesion: 0.24
Nodes (5): main(), solve_all(), catalog(), build_catalog(), example_inputs()

### Community 22 - "Tauri Configuration"
Cohesion: 0.10
Nodes (19): app, security, windows, withGlobalTauri, build, frontendDist, bundle, active (+11 more)

### Community 23 - "solve_nonuniform"
Cohesion: 0.13
Nodes (42): a_blank_form_is_refused_by_name_rather_than_by_division(), a_four_leg_tag_is_rated_on_three_but_a_one_leg_tag_on_one(), a_single_leg_tag_ignores_its_leftover_angle(), a_step_can_lift_a_pick_point_above_hook_level(), asym(), Asym2LegResult, each_crane_is_booked_against_its_own_rating(), effective_tag_angle() (+34 more)

### Community 24 - "matrix_chart"
Cohesion: 0.16
Nodes (20): a_matrix_workbook_is_looked_up_at_the_named_boom(), a_matrix_workbook_parses_to_the_references_own_table(), a_radius_change_re_looks_up_the_capacity_rather_than_keeping_the_old_one(), a_radius_with_no_chartable_capacity_leaves_the_entry_alone(), a_workbook_without_a_title_line_leaves_the_crane_fields_alone(), a_workbook_without_the_new_extras_keeps_them_empty(), boom_changed(), changing_the_radius_clears_the_graphs_published_angle() (+12 more)

### Community 26 - "Bug Report Template"
Cohesion: 0.12
Nodes (16): description, label, options, placeholder, render, value, body, attributes (+8 more)

### Community 27 - "0009 — Taking the dependency refresh"
Cohesion: 0.29
Nodes (6): 0009 — Taking the dependency refresh, Consequences, Decision, Problem, Proposal, What was reviewed

### Community 28 - "Model & Density"
Cohesion: 0.24
Nodes (9): PREVIEW_GENERATION, a_half_typed_line_weighs_zero(), ids_are_unique_and_survive_a_copy(), LiftTotals, next_line_id(), NEXT_LINE_ID, the_required_capacity_is_always_above_the_gross_so_the_over_flag_stays_off(), totals_add_load_and_tackle() (+1 more)

### Community 29 - "Dependabot Config"
Cohesion: 0.12
Nodes (16): patterns, prefix, actions, rust-dependencies, dependency-name, patterns, interval, updates (+8 more)

### Community 30 - "Release Notes Checker"
Cohesion: 0.27
Nodes (12): case_(), download_section(), fail(), main(), self_test(), check-release-notes.sh script, tag_case(), validate_file() (+4 more)

### Community 31 - "LoadType"
Cohesion: 0.22
Nodes (5): LoadType, Fixed, .ORDER, PerMetre, PerVolume

### Community 32 - "Feature Request Template"
Cohesion: 0.14
Nodes (13): description, label, value, body, attributes, id, type, validations (+5 more)

### Community 33 - "Release Workflow Env"
Cohesion: 0.35
Nodes (12): steps, steps, steps, id, if, name, run, shell (+4 more)

### Community 34 - "with"
Cohesion: 0.29
Nodes (7): with, if-no-files-found, key, name, path, persist-credentials, workspaces

### Community 35 - "update.rs"
Cohesion: 0.09
Nodes (34): a_digest_is_lowercase_sha256(), a_malformed_signature_is_refused_not_panicked_on(), a_release_manifest_names_its_installer_by_digest(), a_signature_proves_the_manifest_came_from_the_key(), a_suffix_does_not_count_as_a_version(), check(), CHECK_TIMEOUT, check_update() (+26 more)

### Community 36 - "jobs"
Cohesion: 0.20
Nodes (9): jobs, verify-notes, windows-installers, name, name, runs-on, name, needs (+1 more)

### Community 37 - "Lifting Plan Calculator 2.8.2"
Cohesion: 0.33
Nodes (5): Compatibility and upgrade notes, Download, Lifting Plan Calculator 2.8.2, What is in it, Why it is safe to take

### Community 38 - "Frontend Store & Charts"
Cohesion: 0.32
Nodes (11): createStore(), chartAction(), chooseJib(), emit(), importChartFromFile(), importChartPath(), manualChartEdit(), setBoomAngle() (+3 more)

### Community 39 - "Engineering Council Charter"
Cohesion: 0.39
Nodes (7): Council Review Requirement for Pull Requests, Standing Vetoes (Domain Safety and Application Security), 8-of-10 Council Review Bar, Engineering Council Charter, Solo-Maintainer Review Path, Council Decision Record Template, Domain Safety Engineer Seat

### Community 40 - "pdf.rs"
Cohesion: 0.09
Nodes (46): a_contents_reference_to_an_array_keeps_its_pages(), a_real_webkit_print_keeps_every_page_and_gains_the_footer(), a_rotated_page_is_refused_rather_than_misplaced(), attach_font(), baseline(), baseline_pt(), BASELINE_SHARE, be16() (+38 more)

### Community 41 - "tandem_sections"
Cohesion: 0.41
Nodes (13): capacity_section(), factors_section(), nested_rating_note(), tag_capacity_block(), utilisation_block(), equation(), p(), subheading() (+5 more)

### Community 42 - "Release Notes Process"
Cohesion: 0.18
Nodes (16): Release Cut Process, Draft-First Publish with Immutable Assets, One Release-Notes File per Tag, Council Record 0001: One Release-Notes File per Tag, Release Gate Comparing Tag, Version, Notes and Record, publish-release-notes.sh One-Time Repair Script, Build & Platform Engineer Seat, Council Seats and Rubrics (+8 more)

### Community 44 - "Publish Permissions"
Cohesion: 0.25
Nodes (8): publish, permissions, contents, if, name, needs, permissions, runs-on

### Community 46 - "chart_table"
Cohesion: 0.40
Nodes (4): a_column_workbook_lists_its_own_three_columns_including_the_boom(), a_matrix_workbook_lists_the_column_the_app_is_reading(), chart_table(), the_listed_column_follows_the_boom_selector()

### Community 49 - "AppImage Build Scripts"
Cohesion: 0.25
Nodes (4): APPIMAGE_EXTRACT_AND_RUN, build-appimage-portable.sh script, build-appimage.sh script, make-zsync.sh script

### Community 50 - "Issue Template Config"
Cohesion: 0.33
Nodes (5): blank_issues_enabled, contact_links, about, name, url

### Community 51 - "Tauri Permissions"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 52 - "super"
Cohesion: 0.16
Nodes (16): fixed(), fixed_has_no_grouping(), fmt(), grouped(), NOTHING, nothing_is_an_em_dash(), plain(), thousands_are_separated() (+8 more)

### Community 55 - "Contributing Guide"
Cohesion: 0.28
Nodes (9): Unreleased Changelog Section, Contributing Guide, Conventional Commits, Pull Request Template, Portable Ubuntu 22.04 AppImage Build (glibc 2.35 Floor), GTK 3 / WebKitGTK 4.1 Runtime Bundle (LGPL), Inter Font (SIL OFL 1.1), KaTeX (MIT) (+1 more)

### Community 57 - "Hr"
Cohesion: 0.40
Nodes (5): Hr(), w(), x(), qr(), v()

### Community 60 - "dto.rs"
Cohesion: 0.12
Nodes (19): solve_all(), solve_all_internal(), AppState, Catalog, ChartColumnDto, HitchSpecDto, KATEX_FILES, LoadTypeDto (+11 more)

### Community 61 - "CalcException"
Cohesion: 0.16
Nodes (10): CalcException, chart_lookup_interpolates_between_charted_radii(), chart_lookup_refuses_a_radius_the_chart_does_not_rate(), ChartError, Empty, Negative, NonPositiveCapacity, Outside (+2 more)

### Community 62 - "htmlBuilder"
Cohesion: 0.50
Nodes (3): htmlBuilder(), r, sn()

### Community 65 - "ADR-0001 Rated Capacity Comes Only from a Charted Point"
Cohesion: 0.33
Nodes (4): v2.5.0 Entry: Booked Rating Check and Rated Envelope, ADR-0001 Rated Capacity Comes Only from a Charted Point, Capacity Is a Rating Only at a Specified Radius, ADR-0002 A Plan States Two Checks: the Chart and the Booked Crane

### Community 69 - "LoadChartData"
Cohesion: 0.18
Nodes (21): a_column_workbook_has_no_boom_to_select(), a_declined_suggestion_is_null_rather_than_zero(), applied(), available_booms(), capacity_at(), chart_label(), chart_points(), chart_radii() (+13 more)

### Community 72 - "0002 — A human-friendly plan head"
Cohesion: 0.25
Nodes (8): 0002 — A human-friendly plan head, Consequences, Decision, Proposal, Re-score, Required changes, Scores — first pass, What the council saw

### Community 73 - "0003 — Printed-plan cards, PDF export, footer and working-range diagram"
Cohesion: 0.29
Nodes (7): 0003 — Printed-plan cards, PDF export, footer and working-range diagram, Consequences, Decision, Proposal, Required changes, Scores, What was reviewed

### Community 74 - "0005 — Print the plan with the app's own web view"
Cohesion: 0.12
Nodes (14): 0004 — The AppImage carries its own PDF renderer, Known limits, Proposal, Required changes before merge, Scores, What the council saw, 0005 — Print the plan with the app's own web view, Consequences (+6 more)

### Community 75 - "Lifting Plan Calculator 2.5.2"
Cohesion: 0.33
Nodes (5): Changed, Compatibility and upgrade notes, Download, Highlights, Lifting Plan Calculator 2.5.2

### Community 76 - "Lifting Plan Calculator 2.5.3"
Cohesion: 0.33
Nodes (5): Changed, Compatibility and upgrade notes, Download, Highlights, Lifting Plan Calculator 2.5.3

### Community 77 - "Lifting Plan Calculator 2.5.4"
Cohesion: 0.33
Nodes (5): Compatibility and upgrade notes, Download, Highlights, Lifting Plan Calculator 2.5.4, Notes

### Community 78 - "Changelog"
Cohesion: 0.50
Nodes (4): Changelog, Keep a Changelog + Semantic Versioning, v2.5.1 Entry: Report Head Layout and TL-200M Chart Fix, Release Notes File Shape

### Community 79 - "UsageState"
Cohesion: 0.19
Nodes (14): UsageState, Caution, Over, Within, CraneResult, crane_section(), design_load_from_gross(), escape() (+6 more)

### Community 80 - "Lifting Plan Calculator 2.6.0"
Cohesion: 0.33
Nodes (5): Compatibility and upgrade notes, Download, Highlights, Lifting Plan Calculator 2.6.0, Notes

### Community 81 - "env"
Cohesion: 0.22
Nodes (9): APPIMAGE_EXTRACT_AND_RUN, GH_TOKEN, HAS_SIGNING_KEY, LIBGL_ALWAYS_SOFTWARE, NO_AT_BRIDGE, UPDATE_SIGNING_KEY, WEBKIT_DISABLE_COMPOSITING_MODE, WEBKIT_DISABLE_DMABUF_RENDERER (+1 more)

### Community 82 - "Lifting Plan Calculator 2.8.0"
Cohesion: 0.33
Nodes (5): Compatibility and upgrade notes, Download, Highlights, Lifting Plan Calculator 2.8.0, Notes

### Community 83 - "pdf-export"
Cohesion: 0.50
Nodes (4): pdf-export, name, needs, runs-on

### Community 84 - "0012 — Releases are signed, and the app will not run an update it cannot attribute"
Cohesion: 0.25
Nodes (7): 0012 — Releases are signed, and the app will not run an update it cannot attribute, Decision, Known limits, stated rather than hidden, Problem, Proposal, Scores, What was reviewed

### Community 85 - "Lifting Plan Calculator 2.10.0"
Cohesion: 0.25
Nodes (7): Compatibility and upgrade notes, Download, If you lose the key, Lifting Plan Calculator 2.10.0, Notes, What changed, What this does and does not prove

### Community 86 - "Lifting Plan Calculator 2.8.1"
Cohesion: 0.33
Nodes (5): Compatibility and upgrade notes, Download, Fixes, Lifting Plan Calculator 2.8.1, Notes

### Community 87 - "0010 — The workbook cannot exhaust memory, and Windows opens files without a shell"
Cohesion: 0.25
Nodes (7): 0010 — The workbook cannot exhaust memory, and Windows opens files without a shell, Consequences, Decision, Problem, Proposal, Scores, What was reviewed

### Community 88 - "linux-appimage"
Cohesion: 0.50
Nodes (4): linux-appimage, name, needs, runs-on

### Community 89 - "NonuniformResult"
Cohesion: 0.22
Nodes (4): ensure_nonuniform_variant_marker(), NonuniformResult, Asym, Tandem

### Community 90 - "0011 — The update download is verified, and the build tools are pinned"
Cohesion: 0.25
Nodes (7): 0011 — The update download is verified, and the build tools are pinned, Consequences, Decision, Honest limit, Problem, Proposal, What was reviewed

### Community 91 - "0005 — The workbook names the crane, and the chart is the only check"
Cohesion: 0.29
Nodes (6): 0005 — The workbook names the crane, and the chart is the only check, Known limits, Proposal, Required changes before merge, Scores, What the council saw

### Community 92 - "0006 — A quiet pipeline: audited dependencies, no compiler warnings"
Cohesion: 0.33
Nodes (5): 0006 — A quiet pipeline: audited dependencies, no compiler warnings, Known limits, Proposal, Scores, What the council saw

### Community 93 - "0007 — The last two warnings in the release log"
Cohesion: 0.33
Nodes (5): 0007 — The last two warnings in the release log, Known limits, Proposal, Scores, What the council saw

### Community 94 - "Lifting Plan Calculator 2.9.0"
Cohesion: 0.33
Nodes (5): Compatibility and upgrade notes, Download, Highlights, Lifting Plan Calculator 2.9.0, Notes

### Community 95 - "Lifting Plan Calculator 2.9.1"
Cohesion: 0.33
Nodes (5): Compatibility and upgrade notes, Download, Highlights, Lifting Plan Calculator 2.9.1, Notes

### Community 96 - "Lifting Plan Calculator 2.9.2"
Cohesion: 0.33
Nodes (5): Compatibility and upgrade notes, Download, Fixed, Lifting Plan Calculator 2.9.2, Notes

### Community 97 - "env"
Cohesion: 0.33
Nodes (6): env, CARGO_TERM_COLOR, GIT_CONFIG_COUNT, GIT_CONFIG_KEY_0, GIT_CONFIG_VALUE_0, TAURI_CLI_VERSION

### Community 98 - "ADR-0005 — The load chart is the only capacity check"
Cohesion: 0.40
Nodes (4): ADR-0005 — The load chart is the only capacity check, Consequences, Context, Decision

### Community 99 - "governing_case_words"
Cohesion: 0.67
Nodes (3): governing_case_words(), the_governing_case_is_named_in_words(), governing_words()

### Community 100 - "on"
Cohesion: 0.50
Nodes (4): on, push, workflow_dispatch, tags

### Community 103 - "createPdfViewer"
Cohesion: 0.17
Nodes (27): backingScale(), clampZoom(), fitPage(), fitWidth(), keptPages(), MAX_BACKING, pageLabel(), stepZoom() (+19 more)

### Community 104 - "Lifting Plan Calculator 2.9.4"
Cohesion: 0.29
Nodes (6): Compatibility and upgrade notes, Download, Fixes, Lifting Plan Calculator 2.9.4, Notes, What this does and does not prove

### Community 108 - "Lifting Plan Calculator 2.9.3"
Cohesion: 0.33
Nodes (5): Compatibility and upgrade notes, Download, Fixes, Lifting Plan Calculator 2.9.3, Notes

### Community 191 - "0006 — The report preview is the PDF"
Cohesion: 0.08
Nodes (22): 0006 — The report preview is the PDF, Consequences, Decision, Problem, Proposal, Required changes, Scores, What was reviewed (+14 more)

### Community 227 - "Lifting Plan Calculator 2.7.0"
Cohesion: 0.33
Nodes (5): Compatibility and upgrade notes, Download, Highlights, Lifting Plan Calculator 2.7.0, Notes

## Knowledge Gaps
- **34 isolated node(s):** `lifting-plan-calculator`, `sign-manifest`, `Number`, `N`, `S` (+29 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 575 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **27 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `_` connect `KaTeX Minified Core` to `KaTeX Markup Base`, `Vn`, `KaTeX Parser Internals`, `KaTeX HTML Nodes`, `KaTeX DOM Attributes`, `KaTeX Trusted Types`, `KaTeX DOM Classes`, `KaTeX Font Metrics`, `KaTeX Markup Nodes`, `Hr`, `KaTeX Class Nodes`, `KaTeX Style Nodes`, `htmlBuilder`, `K`?**
  _High betweenness centrality (0.009) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `createGraphScreen()` (e.g. with `radioOption()` and `sync()`) actually correct?**
  _`createGraphScreen()` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `lifting-plan-calculator`, `sign-manifest`, `Number` to the rest of the system?**
  _34 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `el` be split into smaller, more focused modules?**
  _Cohesion score 0.05249033920137398 - nodes in this community are weakly interconnected._
- **Why does `solve_one_leg()` connect `Sling Engine & Tests` to `Report Builder & Load Ratio`, `commands.rs`, `tandem_sections`, `Report Capacity Tests`, `Report Uniform Tests`, `dump_fixtures.rs`, `solve_nonuniform`, `dto.rs`?**
  _High betweenness centrality (0.007) - this node is a cross-community bridge._
- **Are the 12 inferred relationships involving `solve_crane()` (e.g. with `solve_all()` and `solve_all_internal()`) actually correct?**
  _`solve_crane()` has 12 INFERRED edges - model-reasoned connections that need verification._
- **Should `Vn` be split into smaller, more focused modules?**
  _Cohesion score 0.07867494824016563 - nodes in this community are weakly interconnected._