use craftcad_io::model::Point2D;
use craftcad_io::reasons::{AppError, ReasonCode};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine2 {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl Affine2 {
    pub fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    pub fn mul(self, rhs: Self) -> Self {
        Self {
            a: self.a * rhs.a + self.c * rhs.b,
            b: self.b * rhs.a + self.d * rhs.b,
            c: self.a * rhs.c + self.c * rhs.d,
            d: self.b * rhs.c + self.d * rhs.d,
            e: self.a * rhs.e + self.c * rhs.f + self.e,
            f: self.b * rhs.e + self.d * rhs.f + self.f,
        }
    }

    pub fn apply_point(self, p: Point2D) -> Point2D {
        Point2D {
            x: self.a * p.x + self.c * p.y + self.e,
            y: self.b * p.x + self.d * p.y + self.f,
        }
    }

    pub fn angle_rad(self) -> f64 {
        self.b.atan2(self.a)
    }

    pub fn approx_uniform_scale(self, eps: f64) -> Option<f64> {
        let sx = (self.a * self.a + self.b * self.b).sqrt();
        let sy = (self.c * self.c + self.d * self.d).sqrt();
        let dot = self.a * self.c + self.b * self.d;
        if dot.abs() <= eps && (sx - sy).abs() <= eps {
            Some((sx + sy) * 0.5)
        } else {
            None
        }
    }
}

fn skip_ws(s: &str, mut i: usize) -> usize {
    while i < s.len() {
        let ch = s.as_bytes()[i] as char;
        if ch.is_whitespace() || ch == ',' {
            i += 1;
        } else {
            break;
        }
    }
    i
}

fn parse_number(s: &str, i: &mut usize) -> Option<f64> {
    *i = skip_ws(s, *i);
    if *i >= s.len() {
        return None;
    }
    let start = *i;
    let bytes = s.as_bytes();
    let mut j = *i;

    while j < s.len() {
        let ch = bytes[j] as char;
        if ch.is_ascii_digit() || matches!(ch, '+' | '-' | '.' | 'e' | 'E') {
            j += 1;
        } else {
            break;
        }
    }
    if j == start {
        return None;
    }
    *i = j;
    s[start..j].parse::<f64>().ok()
}

fn parse_args(s: &str) -> Vec<f64> {
    let mut i = 0usize;
    let mut out = Vec::new();
    while i < s.len() {
        if let Some(v) = parse_number(s, &mut i) {
            out.push(v);
        } else {
            i += 1;
        }
    }
    out
}

pub fn parse_transform_attr(s: Option<&str>) -> Result<Affine2, AppError> {
    let Some(src) = s else {
        return Ok(Affine2::identity());
    };
    let src = src.trim();
    if src.is_empty() {
        return Ok(Affine2::identity());
    }

    let mut i = 0usize;
    let mut acc = Affine2::identity();

    while i < src.len() {
        i = skip_ws(src, i);
        if i >= src.len() {
            break;
        }

        let name_start = i;
        while i < src.len() {
            let ch = src.as_bytes()[i] as char;
            if ch.is_ascii_alphabetic() {
                i += 1;
            } else {
                break;
            }
        }
        let name = &src[name_start..i];
        i = skip_ws(src, i);
        if i >= src.len() || (src.as_bytes()[i] as char) != '(' {
            return Err(AppError::new(
                ReasonCode::IO_PARSE_SVG_MALFORMED,
                "transform parse failed",
            )
            .with_context("at", i.to_string())
            .with_context("name", name.to_string()));
        }
        i += 1;

        let args_start = i;
        while i < src.len() && (src.as_bytes()[i] as char) != ')' {
            i += 1;
        }
        if i >= src.len() {
            return Err(AppError::new(
                ReasonCode::IO_PARSE_SVG_MALFORMED,
                "transform parse failed (missing ')')",
            )
            .with_context("name", name.to_string()));
        }
        let args_str = &src[args_start..i];
        i += 1;

        let args = parse_args(args_str);
        let m = match name {
            "matrix" if args.len() == 6 => Affine2 {
                a: args[0],
                b: args[1],
                c: args[2],
                d: args[3],
                e: args[4],
                f: args[5],
            },
            "translate" if args.len() == 1 || args.len() == 2 => {
                let tx = args[0];
                let ty = if args.len() == 2 { args[1] } else { 0.0 };
                Affine2 {
                    a: 1.0,
                    b: 0.0,
                    c: 0.0,
                    d: 1.0,
                    e: tx,
                    f: ty,
                }
            }
            "scale" if args.len() == 1 || args.len() == 2 => {
                let sx = args[0];
                let sy = if args.len() == 2 { args[1] } else { args[0] };
                Affine2 {
                    a: sx,
                    b: 0.0,
                    c: 0.0,
                    d: sy,
                    e: 0.0,
                    f: 0.0,
                }
            }
            "rotate" if args.len() == 1 || args.len() == 3 => {
                let ang = args[0].to_radians();
                let (s, c) = ang.sin_cos();
                let r = Affine2 {
                    a: c,
                    b: s,
                    c: -s,
                    d: c,
                    e: 0.0,
                    f: 0.0,
                };
                if args.len() == 3 {
                    let cx = args[1];
                    let cy = args[2];
                    let t1 = Affine2 {
                        a: 1.0,
                        b: 0.0,
                        c: 0.0,
                        d: 1.0,
                        e: cx,
                        f: cy,
                    };
                    let t2 = Affine2 {
                        a: 1.0,
                        b: 0.0,
                        c: 0.0,
                        d: 1.0,
                        e: -cx,
                        f: -cy,
                    };
                    t1.mul(r).mul(t2)
                } else {
                    r
                }
            }
            other => {
                return Err(AppError::new(
                    ReasonCode::IO_PARSE_SVG_MALFORMED,
                    "unsupported transform",
                )
                .with_context("transform", other.to_string())
                .with_context("args", format!("{:?}", args)))
            }
        };

        acc = m.mul(acc);
    }

    Ok(acc)
}
