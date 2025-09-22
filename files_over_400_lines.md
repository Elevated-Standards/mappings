# Files Over 400 Lines

This document lists all files in the workspace that contain more than 400 lines of code.

## Source Code Files (Project Code)

### Rust Files (.rs)

| File | Lines | Description |
|------|-------|-------------|
| `crates/document-parser/src/validation_backup.rs` | 6,368 | Validation backup functionality |
| `crates/document-parser/src/excel.rs` | 4,534 | Excel document processing |
| `crates/document-parser/src/oscal.rs` | 1,563 | OSCAL format handling |
| `crates/document-parser/src/markdown.rs` | 1,521 | Markdown document processing |
| `crates/document-parser/src/fuzzy.rs` | 1,363 | Fuzzy matching algorithms |
| `crates/document-parser/src/validation/poam_validator.rs` | 1,351 | POAM validation logic |
| `crates/document-parser/src/mapping/loader.rs` | 1,076 | Mapping data loader |
| `crates/document-parser/src/word.rs` | 1,074 | Word document processing |
| `crates/document-parser/src/mapping/engine.rs` | 1,033 | Mapping engine core |
| `crates/document-parser/src/mapping/poam_column_mapper.rs` | 911 | POAM column mapping |
| `crates/document-parser/src/validation/reports.rs` | 785 | Validation reporting |
| `crates/document-parser/src/validation/overrides.rs` | 745 | Validation overrides |
| `crates/gap-analysis/src/remediation.rs` | 675 | Gap analysis remediation |
| `crates/document-parser/src/validation/validators.rs` | 671 | Core validators |
| `crates/document-parser/src/mapping/date_converter.rs` | 523 | Date conversion utilities |
| `crates/document-parser/src/validation/confidence.rs` | 515 | Confidence scoring |
| `crates/gap-analysis/src/engine.rs` | 514 | Gap analysis engine |
| `crates/gap-analysis/src/prioritization.rs` | 506 | Priority analysis |
| `crates/fedramp-core/src/models/user.rs` | 497 | User model definitions |
| `crates/document-parser/src/mapping/poam_transformers.rs` | 473 | POAM data transformers |
| `crates/fedramp-core/src/models/audit.rs` | 469 | Audit model definitions |
| `crates/fedramp-core/src/models/system.rs` | 427 | System model definitions |
| `crates/compliance-dashboard/src/widgets.rs` | 419 | Dashboard widgets |
| `crates/gap-analysis/src/baseline.rs` | 409 | Baseline analysis |
| `crates/fedramp-core/src/models/poam.rs` | 408 | POAM model definitions |

### CSS Files

| File | Lines | Description |
|------|-------|-------------|
| `crates/compliance-dashboard/frontend/src/components/UI/AlertBadge.css` | 471 | Alert badge styling |
| `crates/compliance-dashboard/frontend/src/components/UI/ProgressBar.css` | 429 | Progress bar styling |
| `crates/compliance-dashboard/frontend/src/components/UI/Modal.css` | 421 | Modal dialog styling |

## Generated/Build Files

| File | Lines | Description |
|------|-------|-------------|
| `target/debug/build/chrono-tz-870e0f3bd494f42a/out/timezones.rs` | 76,075 | Generated timezone data |
| `target/debug/build/typenum-*/out/tests.rs` | ~20,562 | Generated type-level number tests |
| `target/debug/build/chrono-tz-870e0f3bd494f42a/out/directory.rs` | 701 | Generated timezone directory |

## Dependencies (Node Modules)

The following are the largest dependency files (showing top 20):

| File | Lines | Description |
|------|-------|-------------|
| `crates/compliance-dashboard/frontend/node_modules/typescript/lib/typescript.js` | 199,120 | TypeScript compiler |
| `crates/compliance-dashboard/frontend/node_modules/typescript/lib/_tsc.js` | 132,810 | TypeScript CLI |
| `crates/compliance-dashboard/frontend/node_modules/vite/dist/node/chunks/dep-D5b0Zz6C.js` | 36,299 | Vite bundler |
| `crates/compliance-dashboard/frontend/node_modules/typescript/lib/lib.dom.d.ts` | 29,610 | DOM type definitions |
| `crates/compliance-dashboard/frontend/node_modules/react-dom/cjs/react-dom-profiling.development.js` | 25,380 | React DOM profiling |
| `crates/compliance-dashboard/frontend/node_modules/react-dom/cjs/react-dom-client.development.js` | 24,993 | React DOM client |
| `crates/compliance-dashboard/frontend/node_modules/rollup/dist/es/shared/node-entry.js` | 23,939 | Rollup bundler |
| `crates/compliance-dashboard/frontend/node_modules/rollup/dist/shared/rollup.js` | 23,861 | Rollup core |
| `crates/compliance-dashboard/frontend/node_modules/csstype/index.d.ts` | 21,297 | CSS type definitions |
| `crates/compliance-dashboard/frontend/node_modules/.vite/deps/react-dom_client.js` | 18,106 | Vite processed React DOM |
| `crates/compliance-dashboard/frontend/node_modules/react-dom/cjs/react-dom-profiling.profiling.js` | 16,218 | React DOM profiling |
| `crates/compliance-dashboard/frontend/node_modules/react-dom/cjs/react-dom-client.production.js` | 15,393 | React DOM client (prod) |
| `crates/compliance-dashboard/frontend/node_modules/@babel/parser/lib/index.js` | 14,595 | Babel parser |
| `crates/compliance-dashboard/frontend/node_modules/graphemer/lib/Graphemer.js` | 11,959 | Unicode grapheme library |
| `crates/compliance-dashboard/frontend/node_modules/typescript/lib/typescript.d.ts` | 11,399 | TypeScript type definitions |
| `crates/compliance-dashboard/frontend/node_modules/typescript/lib/lib.webworker.d.ts` | 9,894 | Web Worker type definitions |
| `crates/compliance-dashboard/frontend/node_modules/react-dom/cjs/react-dom-server.edge.development.js` | 9,443 | React DOM server (Edge) |
| `crates/compliance-dashboard/frontend/node_modules/react-dom/cjs/react-dom-server.browser.development.js` | 9,424 | React DOM server (Browser) |
| `crates/compliance-dashboard/frontend/node_modules/react-dom/cjs/react-dom-server.node.development.js` | 9,317 | React DOM server (Node) |
| `crates/compliance-dashboard/frontend/node_modules/rollup/dist/es/shared/watch.js` | 9,297 | Rollup file watcher |

## Summary

- **Total project source files over 400 lines**: 28 files (25 Rust + 3 CSS)
- **Largest project file**: `validation_backup.rs` (6,368 lines)
- **Most files are in**: `document-parser` crate (document processing and validation)
- **Generated files**: Primarily timezone and type-level number generation
- **Dependencies**: Standard web development toolchain (TypeScript, React, Vite, etc.)

## Notes

- Generated files in `target/` directory are build artifacts and can be ignored for code review
- Node modules are third-party dependencies and don't require review
- Focus should be on the 28 project source files for code quality and maintenance
