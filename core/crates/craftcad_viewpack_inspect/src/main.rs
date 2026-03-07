use craftcad_viewpack::{verify_viewpack, VerificationIssueKind, REQUIRED_ARTIFACTS};

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: craftcad-viewpack-inspect <project_path>");
        std::process::exit(2);
    };

    let project = match diycad_project::load(&path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("VIEWPACK_INSPECT_FAIL error={e}");
            std::process::exit(1);
        }
    };

    let Some(vp) = project.viewer_pack_v1 else {
        println!("VIEWPACK_INSPECT_OK status=NOT_GENERATED artifacts=0");
        return;
    };

    let issues = verify_viewpack(&vp);
    let corrupt_count = issues
        .iter()
        .filter(|i| {
            matches!(
                i.kind,
                VerificationIssueKind::HashMismatch | VerificationIssueKind::Base64DecodeFailed
            )
        })
        .count();

    let missing_count = REQUIRED_ARTIFACTS
        .iter()
        .filter(|name| !vp.artifacts.iter().any(|a| a.name == **name))
        .count();

    for name in REQUIRED_ARTIFACTS {
        if vp.artifacts.iter().any(|a| a.name == name) {
            let is_bad = issues.iter().any(|i| {
                i.artifact_name.as_deref() == Some(name)
                    && matches!(
                        i.kind,
                        VerificationIssueKind::HashMismatch
                            | VerificationIssueKind::Base64DecodeFailed
                    )
            });
            if is_bad {
                println!("ARTIFACT {name} STATUS=CORRUPT");
            } else {
                println!("ARTIFACT {name} STATUS=OK");
            }
        } else {
            println!("ARTIFACT {name} STATUS=NOT_GENERATED");
        }
    }

    println!(
        "VIEWPACK_INSPECT_OK status=HAS_PACK artifacts={} missing={} corrupt={} issues={}",
        vp.artifacts.len(),
        missing_count,
        corrupt_count,
        issues.len()
    );
}
