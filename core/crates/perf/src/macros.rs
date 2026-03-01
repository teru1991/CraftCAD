#[macro_export]
macro_rules! perf_span {
    ($name:expr, $body:block) => {{
        let _guard = $crate::timer::perf_span($name);
        $body
    }};
}
