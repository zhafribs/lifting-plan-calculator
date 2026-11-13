# Graph Report - lifting-plan-calculator  (2026-11-13)

## Corpus Check
- 79 files · ~91,969 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 34 file(s) not represented in the graph (top: .woff2 22, (none) 6, .xml 2)
- WARNING: 34 of 170 files seen (20%) have no supported extractor and are missing from this graph entirely. It may look complete while representing only part of the codebase.

## Summary
- 1301 nodes · 3123 edges · 72 communities (53 shown, 19 thin omitted)
- Extraction: 90% EXTRACTED · 10% INFERRED · 0% AMBIGUOUS · INFERRED: 310 edges (avg confidence: 0.84)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Frontend App Shell
- KaTeX Math Internals
- Crane Engine & Tests
- Changelog & Code of Conduct
- Release Notes 2.1.x
- Nonuniform Load Engine
- Graph Screen UI
- Sling Engine & Tests
- Report Builder & Load Ratio
- Tauri Commands
- KaTeX Minified Core
- Excel Cell Data
- CI Workflow
- Calc Engine Primitives
- Excel Unit Rules
- Excel Chart Columns
- Geometry Frontend
- Update Check
- Report Capacity Tests
- Report Uniform Tests
- Report Nonuniform
- XLSX Reader Internals
- Tauri Configuration
- Fixture Dumper
- Excel Lookup Tests
- KaTeX Font Metrics
- Bug Report Template
- Report Sections
- Model & Density
- Dependabot Config
- Release Notes Checker
- DTO & KaTeX Assets
- Feature Request Template
- Release Workflow Env
- Release Action Inputs
- Load Type Enum
- Release Workflow Config
- Hitch Spec DTO
- Frontend Store & Charts
- Calc Exceptions
- Figure Formatting
- Usage Band Logic
- Nonuniform Result DTO
- KaTeX Parser Internals
- Publish Permissions
- Load Measurements
- Chart Table Tests
- KaTeX HTML Nodes
- KaTeX DOM Attributes
- AppImage Build Scripts
- Issue Template Config
- Tauri Permissions
- Chart Action Enum
- KaTeX Trusted Types
- KaTeX DOM Classes
- Section Result Type
- KaTeX Markup Nodes
- KaTeX Parser Helpers
- KaTeX Class Nodes
- KaTeX Style Nodes
- Windows Release Job
- Release Triggers
- KaTeX HTML Builder
- KaTeX Node Base
- KaTeX Markup Base
- Governing Case Words
- Republish Script
- Repository Root

## God Nodes (most connected - your core abstractions)
1. `_` - 58 edges
2. `el()` - 53 edges
3. `solve_crane()` - 49 edges
4. `createGraphScreen()` - 48 edges
5. `LoadChartData` - 35 edges
6. `solve_one_leg()` - 33 edges
7. `fmt()` - 31 edges
8. `createCraneScreen()` - 28 edges
9. `build_report_html()` - 27 edges
10. `Vn` - 27 edges

## Surprising Connections (you probably didn't know these)
- `Pull Request Template` --conceptually_related_to--> `Contributing Guide`  [INFERRED]
  .github/PULL_REQUEST_TEMPLATE.md → CONTRIBUTING.md
- `Draft-First Publish with Immutable Assets` --conceptually_related_to--> `Release Cut Process`  [INFERRED]
  docs/council/decisions/0001-per-tag-release-notes.md → CONTRIBUTING.md
- `Application Security Engineer Seat` --conceptually_related_to--> `Threat Surface: xlsx Reading, Report Write, Embedded Web View`  [INFERRED]
  docs/council/ROLES.md → SECURITY.md
- `Five Calculation Forms Over One Shared Lift` --conceptually_related_to--> `75% Rule Capacity Band`  [INFERRED]
  docs/releases/v2.0.0.md → README.md
- `75% Rule Capacity Band` --conceptually_related_to--> `Capacity Is a Rating Only at a Specified Radius`  [INFERRED]
  README.md → docs/adr/0001-rated-capacity-envelope.md

## Import Cycles
- 2-file cycle: `src-tauri/src/report.rs -> src-tauri/src/report_capacity.rs -> src-tauri/src/report.rs`
- 2-file cycle: `src-tauri/src/report.rs -> src-tauri/src/report_nonuniform.rs -> src-tauri/src/report.rs`
- 2-file cycle: `src-tauri/src/report.rs -> src-tauri/src/report_uniform.rs -> src-tauri/src/report.rs`
- 2-file cycle: `src-tauri/src/calc.rs -> src-tauri/src/model.rs -> src-tauri/src/calc.rs`

## Hyperedges (group relationships)
- **Engineering Council Review Gate** — docs_council_charter_engineering_council, docs_council_charter_eight_of_ten_bar, docs_council_charter_solo_maintainer_gate, docs_council_charter_domain_safety_veto, docs_council_roles_council_seats, docs_council_decisions_0000_template_record_template, contributing_council_review [EXTRACTED 1.00]
- **Per-Tag Release Notes Pipeline** — changelog_changelog, changelog_unreleased_section, contributing_release_process, docs_releases_readme_release_notes_process, docs_releases_readme_notes_file_shape, docs_council_decisions_0001_per_tag_release_notes_one_file_per_tag, docs_council_decisions_0001_per_tag_release_notes_release_gate, docs_council_decisions_0001_per_tag_release_notes_draft_first_publish [EXTRACTED 1.00]
- **Safety Verdict Integrity (Band, Refusal, Booked Check, Scope)** — readme_seventy_five_percent_rule, readme_scope_of_use, docs_adr_0001_rated_capacity_envelope_out_of_chart_refusal, docs_adr_0001_rated_capacity_envelope_charted_point_principle, docs_adr_0002_booked_rating_check_booked_check, docs_council_roles_domain_safety_engineer [INFERRED 0.85]
- **2.1.x wide-window layout rework (scroll panes, split cut, rail footer)** — docs_releases_v210, docs_releases_v211, docs_releases_v212, docs_releases_v213, docs_releases_v214, docs_releases_v215, docs_releases_v210_independent_scroll_panes, docs_releases_v214_two_column_layout, docs_releases_v210_rail_footer [EXTRACTED 1.00]
- **Crane load-chart correctness: workbook import, 75% rule, booked rating, refusal** — docs_releases_v230, docs_releases_v250, docs_releases_v251, docs_releases_v230_workbook_crane_position, docs_releases_v230_seventy_five_percent_rule, docs_releases_v250_booked_crane_rating, docs_releases_v250_radius_refusal, docs_releases_v250_unit_detection [EXTRACTED 1.00]
- **Cross-platform distribution and update pipeline** — docs_releases_v220, docs_releases_v240, docs_releases_v220_windows_build, docs_releases_v220_appimage_delta_updates, docs_releases_v220_release_asset_naming, docs_releases_v240_update_reminders, docs_releases_v240_windows_install_action, docs_releases_v220_glibc_floor [EXTRACTED 1.00]

## Communities (72 total, 19 thin omitted)

### Community 0 - "Frontend App Shell"
Cohesion: 0.06
Nodes (94): api, errorText(), main(), go(), runInstaller(), showReport(), showUpdate(), toast() (+86 more)

### Community 1 - "KaTeX Math Internals"
Cohesion: 0.08
Nodes (6): Cr, Hn, Mn, Nr, Vn, zn

### Community 2 - "Crane Engine & Tests"
Cohesion: 0.08
Nodes (52): a_booked_rating_that_clears_the_requirement_leaves_the_chart_verdict(), a_crane_booked_below_the_required_capacity_fails_on_the_booking(), a_hand_typed_line_carries_all_four_of_its_columns_into_the_result(), a_jib_angle_is_implied_from_the_radius_for_the_lower_solution(), a_jib_angle_outside_the_jib_sheet_is_refused_with_its_own_range(), a_jib_configuration_derives_the_working_radius_from_the_boom_angle(), a_jib_configuration_is_checked_against_the_jib_table_at_the_boom_angle(), a_jib_offset_is_clockwise_and_raises_the_boom_for_the_same_reach() (+44 more)

### Community 3 - "Changelog & Code of Conduct"
Cohesion: 0.06
Nodes (55): Changelog, Keep a Changelog + Semantic Versioning, Unreleased Changelog Section, v2.5.0 Entry: Booked Rating Check and Rated Envelope, v2.5.1 Entry: Report Head Layout and TL-200M Chart Fix, Contributor Covenant Code of Conduct 2.1, Community Impact Enforcement Ladder, Contributing Guide (+47 more)

### Community 4 - "Release Notes 2.1.x"
Cohesion: 0.06
Nodes (53): Lifting Plan Calculator 2.1.0 release notes, Independent scroll panes on wide windows, Rail footer pinned beneath the icon rail, Scope of use — calculation aid, not a certified lift plan, Always-visible working range diagram (Graph tab), Lifting Plan Calculator 2.1.1 release notes, Lifting Plan Calculator 2.1.2 release notes, Overall Weight tab (+45 more)

### Community 5 - "Nonuniform Load Engine"
Cohesion: 0.13
Nodes (38): a_blank_form_is_refused_by_name_rather_than_by_division(), a_four_leg_tag_is_rated_on_three_but_a_one_leg_tag_on_one(), a_single_leg_tag_ignores_its_leftover_angle(), a_step_can_lift_a_pick_point_above_hook_level(), each_crane_is_booked_against_its_own_rating(), effective_tag_angle(), err(), every_scenario_in_the_registry_can_be_solved() (+30 more)

### Community 6 - "Graph Screen UI"
Cohesion: 0.12
Nodes (41): createGraphScreen(), apply(), applyWorkbook(), checkboxOption(), drawAngleArc(), drawAxesGround(), drawBoom(), drawCrane() (+33 more)

### Community 7 - "Sling Engine & Tests"
Cohesion: 0.11
Nodes (34): a_bridle_is_rated_on_one_leg_fewer_than_it_has(), a_missing_shape_measurement_is_named_rather_than_treated_as_zero(), a_one_leg_sling_is_never_derated_by_a_leftover_tag_angle(), a_rectangular_choke_flare_must_stay_inside_the_vertical(), a_sling_too_short_for_the_wrap_names_the_length_it_needs(), a_two_leg_sling_does_take_its_rated_angle(), AF_DECIMALS, an_arrangement_that_cannot_reach_the_hook_says_so() (+26 more)

### Community 8 - "Report Builder & Load Ratio"
Cohesion: 0.10
Nodes (36): DESIGN_LOAD_RATIO, a_blank_crane_says_not_entered_rather_than_zero(), a_chart_with_no_capacity_omits_the_usage_equation(), a_crane_check_that_could_not_be_made_is_named_in_the_plan(), a_null_figure_is_not_a_zero(), a_plan_with_no_details_still_prints_the_blank_lines(), a_section_is_gated_on_the_include_set(), a_tabs_report_carries_its_own_section_and_the_crane() (+28 more)

### Community 9 - "Tauri Commands"
Cohesion: 0.11
Nodes (23): app_info(), AppInfo, build_report(), chart_update(), ChartHolder, ChartUpdate, import_chart(), open_report_file() (+15 more)

### Community 10 - "KaTeX Minified Core"
Cohesion: 0.09
Nodes (15): _, ct(), $e(), handler(), It(), je(), mt(), N() (+7 more)

### Community 11 - "Excel Cell Data"
Cohesion: 0.16
Nodes (27): CellValue, Number, Text, chart_number(), declared_unit(), detect_header_row(), detect_matrix(), detect_unit() (+19 more)

### Community 12 - "CI Workflow"
Cohesion: 0.07
Nodes (26): concurrency, cancel-in-progress, group, env, CARGO_TERM_COLOR, jobs, test, name (+18 more)

### Community 13 - "Calc Engine Primitives"
Cohesion: 0.18
Nodes (22): a_zero_quantity_line_still_carries_its_weight_on_a_load_line_but_not_on_tackle(), CalcResult, chart_lookup_interpolates_between_charted_radii(), chart_lookup_refuses_a_radius_the_chart_does_not_rate(), chart_usage_percent(), clamp_qty(), CRANE_USAGE_RATIO, design_weight() (+14 more)

### Community 14 - "Excel Unit Rules"
Cohesion: 0.14
Nodes (26): a_boom_table_lists_only_its_own_charted_radii(), a_boom_with_no_charted_rows_blanks_the_mirror_instead_of_keeping_the_old_one(), a_chart_that_declares_nothing_still_falls_back_to_its_figures(), a_column_chart_takes_the_unit_from_its_title_when_the_header_is_silent(), a_declared_kg_chart_is_not_read_as_tonnes(), a_declared_ton_chart_keeps_tonnes_even_when_the_figures_are_large(), a_file_that_is_not_a_workbook_is_refused_in_words_an_operator_can_act_on(), a_workbook_carries_its_position_and_jib_tables_into_the_chart() (+18 more)

### Community 15 - "Excel Chart Columns"
Cohesion: 0.16
Nodes (23): a_column_workbook_has_no_boom_to_select(), a_column_workbook_parses_to_the_references_own_table(), applied(), available_booms(), boom_changed(), chart_label(), chart_points(), chart_radii() (+15 more)

### Community 16 - "Geometry Frontend"
Cohesion: 0.15
Nodes (23): ANGLE_MAX, ANGLE_MIN, angleForTip(), angleForWorkingRadius(), assemblyTip(), boomTip(), clamp(), CONFIG_BOOM (+15 more)

### Community 17 - "Update Check"
Cohesion: 0.13
Nodes (19): a_suffix_does_not_count_as_a_version(), check(), CHECK_TIMEOUT, check_update(), DOWNLOAD_PREFIX, DOWNLOAD_TIMEOUT, install(), install_update() (+11 more)

### Community 18 - "Report Capacity Tests"
Cohesion: 0.18
Nodes (23): a_nested_lift_names_both_slings_throughout(), a_single_leg_prints_no_tag_capacity_block(), CHOKE_KEYS, count_h2(), direct_hook(), every_arrangement_carries_the_tag_rating_rule_when_it_has_more_than_one_leg(), label_index(), nested3() (+15 more)

### Community 19 - "Report Uniform Tests"
Cohesion: 0.16
Nodes (19): a_nested_input_section_names_its_piles_and_its_second_sling(), a_two_leg_nested_system_has_no_pile_row(), every_arrangement_produces_a_geometry_section_of_its_own(), geometry(), inputs(), nested_height_caption(), secondary_group_labels(), secondary_labels() (+11 more)

### Community 20 - "Report Nonuniform"
Cohesion: 0.16
Nodes (20): TandemResult, escape(), label_value(), a_block_that_was_never_set_up_says_which_tab_to_complete(), a_crane_with_no_chart_capacity_says_so_rather_than_showing_zero_percent(), a_missing_tag_says_the_check_was_not_done_rather_than_passing_it(), all_three_cases_are_worked_and_the_vertical_one_is_not_skipped(), asym_html() (+12 more)

### Community 21 - "XLSX Reader Internals"
Cohesion: 0.11
Nodes (11): COLUMN_RULES, DEFAULT_MAX_ROWS, jib_options(), MAX_CHART_BYTES, NOT_A_WORKBOOK, pair_from_cell(), pair_from_text(), suggest_jib_angle() (+3 more)

### Community 22 - "Tauri Configuration"
Cohesion: 0.11
Nodes (18): app, security, windows, withGlobalTauri, build, frontendDist, bundle, active (+10 more)

### Community 23 - "Fixture Dumper"
Cohesion: 0.18
Nodes (13): main(), solve_all(), catalog(), build_catalog(), asym(), Asym2LegResult, example_nonuniform_inputs(), the_asymmetric_bridle_matches_the_reference() (+5 more)

### Community 24 - "Excel Lookup Tests"
Cohesion: 0.17
Nodes (18): a_declined_suggestion_is_null_rather_than_zero(), a_matrix_workbook_is_looked_up_at_the_named_boom(), a_matrix_workbook_parses_to_the_references_own_table(), a_radius_change_re_looks_up_the_capacity_rather_than_keeping_the_old_one(), a_radius_with_no_chartable_capacity_leaves_the_entry_alone(), a_workbook_without_the_new_extras_keeps_them_empty(), changing_the_radius_clears_the_graphs_published_angle(), changing_the_working_radius_writes_through_to_the_entry() (+10 more)

### Community 26 - "Bug Report Template"
Cohesion: 0.12
Nodes (16): description, label, options, placeholder, render, value, body, attributes (+8 more)

### Community 27 - "Report Sections"
Cohesion: 0.38
Nodes (17): capacity_section(), factors_section(), installed_tag_block(), nonuniform_rating_legs_note(), tag_capacity_block(), utilisation_block(), equation(), heading() (+9 more)

### Community 28 - "Model & Density"
Cohesion: 0.21
Nodes (9): DEFAULT_DENSITY_KG_M3, gross_lift_weight(), a_half_typed_line_weighs_zero(), ids_are_unique_and_survive_a_copy(), next_line_id(), NEXT_LINE_ID, the_required_capacity_is_always_above_the_gross_so_the_over_flag_stays_off(), totals_add_load_and_tackle() (+1 more)

### Community 29 - "Dependabot Config"
Cohesion: 0.13
Nodes (14): patterns, prefix, actions, rust-dependencies, patterns, interval, updates, commit-message (+6 more)

### Community 30 - "Release Notes Checker"
Cohesion: 0.27
Nodes (12): case_(), download_section(), fail(), main(), self_test(), check-release-notes.sh script, tag_case(), validate_file() (+4 more)

### Community 31 - "DTO & KaTeX Assets"
Cohesion: 0.22
Nodes (12): Catalog, ChartColumnDto, KATEX_FILES, LoadTypeDto, TandemMethodDto, TackleItem, NonuniformInputs, SCENARIO_SPECS (+4 more)

### Community 32 - "Feature Request Template"
Cohesion: 0.14
Nodes (13): description, label, value, body, attributes, id, type, validations (+5 more)

### Community 33 - "Release Workflow Env"
Cohesion: 0.26
Nodes (14): APPIMAGE_EXTRACT_AND_RUN, GH_TOKEN, steps, steps, env, id, if, name (+6 more)

### Community 34 - "Release Action Inputs"
Cohesion: 0.14
Nodes (14): with, body_path, draft, fail_on_unmatched_files, files, if-no-files-found, key, merge-multiple (+6 more)

### Community 35 - "Load Type Enum"
Cohesion: 0.14
Nodes (6): LoadType, Fixed, .ORDER, PerMetre, PerVolume, LoadItem

### Community 36 - "Release Workflow Config"
Cohesion: 0.15
Nodes (12): env, CARGO_TERM_COLOR, TAURI_CLI_VERSION, jobs, linux-appimage, verify-notes, name, needs (+4 more)

### Community 37 - "Hitch Spec DTO"
Cohesion: 0.27
Nodes (10): HitchSpecDto, nested_rating_note(), notes_section(), configuration_label(), dimension_rows(), section_inputs(), HITCH_SPECS, HitchSpec (+2 more)

### Community 38 - "Frontend Store & Charts"
Cohesion: 0.32
Nodes (11): createStore(), chartAction(), chooseJib(), emit(), importChartFromFile(), importChartPath(), manualChartEdit(), setBoomAngle() (+3 more)

### Community 39 - "Calc Exceptions"
Cohesion: 0.24
Nodes (6): CalcException, ChartError, Empty, Negative, NonPositiveCapacity, Outside

### Community 40 - "Figure Formatting"
Cohesion: 0.33
Nodes (9): fixed(), fixed_has_no_grouping(), fmt(), grouped(), NOTHING, nothing_is_an_em_dash(), plain(), thousands_are_separated() (+1 more)

### Community 41 - "Usage Band Logic"
Cohesion: 0.22
Nodes (6): crane_usage_band(), usage_bands_match_the_source(), UsageState, Caution, Over, Within

### Community 42 - "Nonuniform Result DTO"
Cohesion: 0.22
Nodes (4): ensure_nonuniform_variant_marker(), NonuniformResult, Asym, Tandem

### Community 44 - "Publish Permissions"
Cohesion: 0.25
Nodes (8): publish, permissions, contents, if, name, needs, permissions, runs-on

### Community 46 - "Chart Table Tests"
Cohesion: 0.29
Nodes (6): a_column_workbook_lists_its_own_three_columns_including_the_boom(), a_matrix_workbook_lists_the_column_the_app_is_reading(), chart_table(), err(), ExcelImportException, the_listed_column_follows_the_boom_selector()

### Community 49 - "AppImage Build Scripts"
Cohesion: 0.33
Nodes (3): build-appimage-portable.sh script, build-appimage.sh script, make-zsync.sh script

### Community 50 - "Issue Template Config"
Cohesion: 0.33
Nodes (5): blank_issues_enabled, contact_links, about, name, url

### Community 51 - "Tauri Permissions"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 52 - "Chart Action Enum"
Cohesion: 0.33
Nodes (6): ChartAction, Boom, Forget, Jib, Manual, Radius

### Community 57 - "KaTeX Parser Helpers"
Cohesion: 0.40
Nodes (5): Hr(), w(), x(), qr(), v()

### Community 60 - "Windows Release Job"
Cohesion: 0.50
Nodes (4): windows-installers, name, needs, runs-on

### Community 61 - "Release Triggers"
Cohesion: 0.50
Nodes (4): on, push, workflow_dispatch, tags

### Community 62 - "KaTeX HTML Builder"
Cohesion: 0.50
Nodes (3): htmlBuilder(), r, sn()

### Community 65 - "Governing Case Words"
Cohesion: 0.67
Nodes (3): governing_case_words(), the_governing_case_is_named_in_words(), governing_words()

## Knowledge Gaps
- **32 isolated node(s):** `lifting-plan-calculator`, `Fixed`, `PerMetre`, `PerVolume`, `Empty` (+27 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 330 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **19 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `solve_crane()` connect `Crane Engine & Tests` to `Report Builder & Load Ratio`, `Tauri Commands`, `Usage Band Logic`, `Calc Engine Primitives`, `Fixture Dumper`, `Excel Lookup Tests`?**
  _High betweenness centrality (0.016) - this node is a cross-community bridge._
- **Are the 12 inferred relationships involving `solve_crane()` (e.g. with `solve_all()` and `solve_all_internal()`) actually correct?**
  _`solve_crane()` has 12 INFERRED edges - model-reasoned connections that need verification._
- **What connects `lifting-plan-calculator`, `Fixed`, `PerMetre` to the rest of the system?**
  _32 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Frontend App Shell` be split into smaller, more focused modules?**
  _Cohesion score 0.06260504201680672 - nodes in this community are weakly interconnected._
- **Why does `_` connect `KaTeX Minified Core` to `KaTeX Markup Base`, `KaTeX Math Internals`, `KaTeX Parser Internals`, `KaTeX HTML Nodes`, `KaTeX DOM Attributes`, `KaTeX Trusted Types`, `KaTeX DOM Classes`, `KaTeX Font Metrics`, `KaTeX Markup Nodes`, `KaTeX Parser Helpers`, `KaTeX Class Nodes`, `KaTeX Style Nodes`, `KaTeX HTML Builder`, `KaTeX Node Base`?**
  _High betweenness centrality (0.011) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `createGraphScreen()` (e.g. with `radioOption()` and `sync()`) actually correct?**
  _`createGraphScreen()` has 2 INFERRED edges - model-reasoned connections that need verification._
- **Should `KaTeX Math Internals` be split into smaller, more focused modules?**
  _Cohesion score 0.07867494824016563 - nodes in this community are weakly interconnected._