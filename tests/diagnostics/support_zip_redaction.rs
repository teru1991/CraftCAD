use craftcad_diagnostics::SupportZipBuilder;
use craftcad_security::ConsentState;

#[test]
fn support_zip_redaction_smoke() {
    let out = std::env::temp_dir().join("craftcad_support_test.zip");
    let _ = SupportZipBuilder::new()
        .attach_consent(ConsentState::default())
        .build(&out)
        .unwrap();
}
