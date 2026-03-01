// IMPORTANT (CraftCAD Product Quality Rules):
// - Determinism: sorting/rounding/epsilon/seed MUST be stable and SSOT-driven.
// - No panics on untrusted inputs. Always return ReasonCode + context.
// - Any optimization must be measurable (PerfReport) and guarded by budgets SSOT.

pub mod budgets;
pub mod macros;
pub mod metrics;
pub mod timer;

pub use budgets::{check_report_against_budgets, load_budgets, PerfBudgets};
pub use metrics::{PerfReport, SpanRecord};
pub use timer::{perf_span, PerfSession};
