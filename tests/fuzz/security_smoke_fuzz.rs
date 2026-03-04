use security::{ExternalRefPolicy, Limits, LimitsProfile, PathValidationContext, Sandbox};

fn xorshift64(mut x: u64) -> impl FnMut() -> u64 {
    move || {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x
    }
}

fn gen_path(mut rnd: impl FnMut() -> u64) -> String {
    let mut s = String::new();
    let segs = (rnd() % 8) + 1;
    for i in 0..segs {
        if i > 0 {
            s.push('/');
        }
        let t = rnd() % 6;
        match t {
            0 => s.push_str(".."),
            1 => s.push('.'),
            2 => s.push_str("C:"),
            3 => {}
            _ => {
                let len = (rnd() % 12) + 1;
                for _ in 0..len {
                    let c = (b'a' + (rnd() % 26) as u8) as char;
                    s.push(c);
                }
            }
        }
    }
    s
}

#[test]
fn security_smoke_fuzz_does_not_panic() {
    let limits = Limits::load_from_ssot(LimitsProfile::Default).unwrap();
    let sb = Sandbox::new(ExternalRefPolicy::Reject);
    let ctx = PathValidationContext {
        max_depth: limits.max_path_depth,
    };

    let mut rnd = xorshift64(0x1234_5678_9abc_def0);
    for _ in 0..2000 {
        let p = gen_path(&mut rnd);
        let _ = sb.normalize_rel_path(ctx.clone(), &p);
        let _ = sb.reject_external_ref(&p);
    }
}
