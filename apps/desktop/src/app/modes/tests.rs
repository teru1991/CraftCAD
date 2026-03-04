use super::*;

#[test]
fn same_event_same_result() {
    let mut m = ModesController::new().unwrap();
    let s0 = m.state.clone();
    let r1 = m.apply(transitions::ModeEvent::KeyEsc);

    let mut m2 = ModesController::new().unwrap();
    m2.state = s0;
    let r2 = m2.apply(transitions::ModeEvent::KeyEsc);

    assert_eq!(
        r1.new_state.current_mode.as_str(),
        r2.new_state.current_mode.as_str()
    );
}

#[test]
fn job_running_denies_draw() {
    let mut m = ModesController::new().unwrap();
    m.state.job_running = true;
    let r = m.apply(transitions::ModeEvent::ToolSelectDraw);
    assert!(r.denied.is_some());
}
