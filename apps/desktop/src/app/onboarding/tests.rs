use super::*;
use std::collections::BTreeMap;

struct MemKv(std::collections::BTreeMap<String, String>);

impl first_run::KvStore for MemKv {
    fn get_string(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }

    fn set_string(&mut self, key: &str, value: String) -> Result<(), String> {
        self.0.insert(key.to_string(), value);
        Ok(())
    }
}

struct FakeOpLog {
    ops: Vec<(String, BTreeMap<String, serde_yaml::Value>)>,
}

impl completion::OpLogQuery for FakeOpLog {
    fn has_op(&self, op_kind: &str, args: &BTreeMap<String, serde_yaml::Value>) -> bool {
        self.ops.iter().any(|(k, a)| k == op_kind && a == args)
    }
}

struct FakeJobs {
    st: std::collections::BTreeMap<String, completion::JobStatus>,
}

impl completion::JobQuery for FakeJobs {
    fn job_status(&self, job_kind: &str) -> completion::JobStatus {
        self.st
            .get(job_kind)
            .cloned()
            .unwrap_or(completion::JobStatus::Unknown)
    }
}

#[test]
fn tutorial_advances_when_required_met() {
    let flow = spec::OnboardingFlowSsot {
        version: 1,
        flow_id: "onboarding_sample_to_print".to_string(),
        entrypoints: vec!["first_run_auto".to_string()],
        completion: spec::Completion { all_of: vec![] },
        steps: vec![spec::StepSpec {
            id: "open_sample".to_string(),
            title_key: "t1".to_string(),
            body_key: "b1".to_string(),
            required: spec::RequiredExpr {
                any_of: vec![spec::ReqAtom::Op {
                    op: "OpenSample".to_string(),
                    args: Some(BTreeMap::from([(
                        "sample_id".to_string(),
                        serde_yaml::Value::String("sample_shelf_project".to_string()),
                    )])),
                }],
            },
            next: "done".to_string(),
            can_skip: true,
            links: None,
        }],
        policy: spec::Policy {
            allow_rerun: true,
            allow_skip: true,
            sample_open_mode: "read_only".to_string(),
            recommended_actions: vec![],
        },
    };

    let model = tutorial_state::TutorialModel::from_ssot(&flow);
    let mut state = tutorial_state::TutorialState::new(&model);
    let engine = completion::CompletionEngine::new(flow);

    let oplog = FakeOpLog { ops: vec![] };
    let jobs = FakeJobs {
        st: Default::default(),
    };
    engine.tick(&model, &mut state, &oplog, &jobs, 123);
    assert!(!state.is_done);

    let oplog2 = FakeOpLog {
        ops: vec![(
            "OpenSample".to_string(),
            BTreeMap::from([(
                "sample_id".to_string(),
                serde_yaml::Value::String("sample_shelf_project".to_string()),
            )]),
        )],
    };
    engine.tick(&model, &mut state, &oplog2, &jobs, 124);
    assert!(state.is_done);
}

#[test]
fn first_run_store_is_deterministic() {
    let kv = Box::new(MemKv(Default::default()));
    let mut fr = first_run::FirstRun::new(kv);
    assert!(fr.should_auto_start());
    fr.mark_started(100).unwrap();
    let st = fr.load();
    assert!(st.has_seen_onboarding);
    assert_eq!(st.last_started_unix_ms, Some(100));

    fr.mark_completed(200).unwrap();
    let st2 = fr.load();
    assert!(st2.completed);
    assert_eq!(st2.last_finished_unix_ms, Some(200));
}
