# Implementation Plan: [FEATURE]

**Branch**: `[###-feature-name]` | **Date**: [DATE] | **Spec**: [link]
**Input**: Feature specification from `/specs/[###-feature-name]/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

[Extract from feature spec: primary requirement + technical approach from research]

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: [e.g., Python 3.11, Swift 5.9, Rust 1.75 or NEEDS CLARIFICATION]  
**Primary Dependencies**: [e.g., FastAPI, UIKit, LLVM or NEEDS CLARIFICATION]  
**Storage**: [if applicable, e.g., PostgreSQL, CoreData, files or N/A]  
**Testing**: [e.g., pytest, XCTest, cargo test or NEEDS CLARIFICATION]  
**Target Platform**: [e.g., Linux server, iOS 15+, WASM or NEEDS CLARIFICATION]
**Project Type**: [single/web/mobile - determines source structure]  
**Performance Goals**: [domain-specific, e.g., 1000 req/s, 10k lines/sec, 60 fps or NEEDS CLARIFICATION]  
**Constraints**: [domain-specific, e.g., <200ms p95, <100MB memory, offline-capable or NEEDS CLARIFICATION]  
**Scale/Scope**: [domain-specific, e.g., 10k users, 1M LOC, 50 screens or NEEDS CLARIFICATION]

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

[Gates determined based on constitution file]

### Architecture & API Design Review

**SDK/API Usability** (if this feature exposes public APIs):
- [ ] APIs minimize boilerplate - provide convenience functions for common operations
- [ ] Sensible defaults reduce required configuration
- [ ] Method names follow principle of least surprise
- [ ] Interface design makes incorrect usage difficult
- [ ] Both low-level control and high-level convenience provided

**Separation of Concerns**:
- [ ] Data structures remain pure (no runtime dependencies)
- [ ] Configuration types only handle parsing/validation
- [ ] Runtime instantiation at module boundaries or associated functions
- [ ] No conceptual cycles (data → runtime, not bidirectional)
- [ ] Clear dependency graph maintained

**Test Organization** (per Constitution IV):
- [ ] Unit tests planned to be co-located in source files (`#[cfg(test)] mod tests`)
- [ ] Integration tests planned for `tests/` directory
- [ ] No standalone unit test files in `tests/` directory
- [ ] Mocking strategy defined (simple in source, comprehensive in `tests/common/`)
- [ ] Clear boundary between unit (single module) and integration (cross-module) tests

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```text
# [REMOVE IF UNUSED] Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── common/              # Shared test utilities and comprehensive mocks
├── integration/         # Cross-module integration tests
└── contract/            # Contract/compliance tests
# NOTE: Unit tests live in src/ files as #[cfg(test)] mod tests

# [REMOVE IF UNUSED] Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# [REMOVE IF UNUSED] Option 3: Mobile + API (when "iOS/Android" detected)
api/
└── [same as backend above]

ios/ or android/
└── [platform-specific structure: feature modules, UI flows, platform tests]
```

**Structure Decision**: [Document the selected structure and reference the real
directories captured above]

## Feature Finalization Requirement

**Constitutional Mandate**: Per RANCH Constitution v1.4.0, Section VII, this feature MUST include formal finalization before being marked production-ready.

### Required Deliverables for Feature Completion

1. **FEATURE_COMPLETE.md** in `specs/[###-feature-name]/` folder containing:
   - Executive summary of delivered functionality
   - Complete list of user stories and task counts
   - Test results (total tests, coverage percentage, pass rate)
   - Validation evidence for all acceptance criteria
   - Known limitations or scope reductions
   - Production readiness checklist
   - Clear status declaration (COMPLETE/BLOCKED/DEFERRED)

2. **tasks.md** with all tasks marked complete ([X]) or incomplete ([ ] with reason)

3. **IMPLEMENTATION_PLAN.md** (or equivalent) updated with feature completion status

4. **CHANGELOG.md** entry documenting user-facing changes

5. **Traceability** from specification → implementation → tests → documentation

### Finalization Checklist

Before declaring feature COMPLETE, verify:
- [ ] All user story acceptance criteria met and documented
- [ ] Test results captured and meet quality thresholds
- [ ] FEATURE_COMPLETE.md created with all required sections
- [ ] All implementation artifacts traceable to specification
- [ ] Feature status easily discoverable by team members
- [ ] Known limitations explicitly documented
- [ ] Production deployment readiness assessed

**Note**: Feature finalization is not optional. It ensures institutional knowledge capture, enables status discovery, and verifies production readiness before release.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
