use craftcad_perf::{check_report_against_budgets, load_budgets, perf_span, PerfSession};

#[test]
fn perf_smoke_budget_check() {
    let session = PerfSession::start("sample_board_hole_chamfer_v1");
    perf_span!("open", { std::thread::sleep(std::time::Duration::from_millis(1)); });
    let report = session.finish();
    let budgets = load_budgets("docs/specs/perf/budgets.json").expect("load budgets");
    let _errs = check_report_against_budgets(&report, &budgets);
}
