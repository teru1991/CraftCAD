use craftcad_serialize::{Geom2D, Reason, ReasonCode, Result, Vec2};

const MIN_SCALE: f64 = 1e-12;

fn valid(v: f64) -> bool {
    v.is_finite()
}

fn valid_vec2(v: &Vec2) -> bool {
    valid(v.x) && valid(v.y)
}

fn add(v: &Vec2, dx: f64, dy: f64) -> Vec2 {
    Vec2 {
        x: v.x + dx,
        y: v.y + dy,
    }
}

fn rotate_point(p: &Vec2, center: &Vec2, angle: f64) -> Vec2 {
    let c = angle.cos();
    let s = angle.sin();
    let x = p.x - center.x;
    let y = p.y - center.y;
    Vec2 {
        x: center.x + x * c - y * s,
        y: center.y + x * s + y * c,
    }
}

fn scale_point(p: &Vec2, center: &Vec2, sx: f64, sy: f64) -> Vec2 {
    Vec2 {
        x: center.x + (p.x - center.x) * sx,
        y: center.y + (p.y - center.y) * sy,
    }
}

pub fn translate(geom: &Geom2D, dx: f64, dy: f64) -> Result<Geom2D> {
    if !valid(dx) || !valid(dy) {
        return Err(Reason::from_code(ReasonCode::EditInvalidNumeric));
    }
    Ok(match geom {
        Geom2D::Line { a, b } => Geom2D::Line {
            a: add(a, dx, dy),
            b: add(b, dx, dy),
        },
        Geom2D::Circle { c, r } => Geom2D::Circle {
            c: add(c, dx, dy),
            r: *r,
        },
        Geom2D::Arc {
            c,
            r,
            start_angle,
            end_angle,
            ccw,
        } => Geom2D::Arc {
            c: add(c, dx, dy),
            r: *r,
            start_angle: *start_angle,
            end_angle: *end_angle,
            ccw: *ccw,
        },
        Geom2D::Polyline { pts, closed } => Geom2D::Polyline {
            pts: pts.iter().map(|p| add(p, dx, dy)).collect(),
            closed: *closed,
        },
    })
}

pub fn rotate(geom: &Geom2D, center: &Vec2, angle_rad: f64) -> Result<Geom2D> {
    if !valid(angle_rad) || !valid_vec2(center) {
        return Err(Reason::from_code(ReasonCode::EditInvalidNumeric));
    }
    Ok(match geom {
        Geom2D::Line { a, b } => Geom2D::Line {
            a: rotate_point(a, center, angle_rad),
            b: rotate_point(b, center, angle_rad),
        },
        Geom2D::Circle { c, r } => Geom2D::Circle {
            c: rotate_point(c, center, angle_rad),
            r: *r,
        },
        Geom2D::Arc {
            c,
            r,
            start_angle,
            end_angle,
            ccw,
        } => Geom2D::Arc {
            c: rotate_point(c, center, angle_rad),
            r: *r,
            start_angle: *start_angle + angle_rad,
            end_angle: *end_angle + angle_rad,
            ccw: *ccw,
        },
        Geom2D::Polyline { pts, closed } => Geom2D::Polyline {
            pts: pts
                .iter()
                .map(|p| rotate_point(p, center, angle_rad))
                .collect(),
            closed: *closed,
        },
    })
}

pub fn scale(geom: &Geom2D, center: &Vec2, sx: f64, sy: f64) -> Result<Geom2D> {
    if !valid(sx) || !valid(sy) || !valid_vec2(center) {
        return Err(Reason::from_code(ReasonCode::EditInvalidNumeric));
    }
    if sx.abs() <= MIN_SCALE || sy.abs() <= MIN_SCALE {
        return Err(Reason::from_code(ReasonCode::EditTransformWouldDegenerate));
    }
    Ok(match geom {
        Geom2D::Line { a, b } => Geom2D::Line {
            a: scale_point(a, center, sx, sy),
            b: scale_point(b, center, sx, sy),
        },
        Geom2D::Circle { c, r } => {
            if (sx.abs() - sy.abs()).abs() > MIN_SCALE {
                return Err(Reason::from_code(ReasonCode::EditTransformWouldDegenerate));
            }
            Geom2D::Circle {
                c: scale_point(c, center, sx, sy),
                r: *r * sx.abs(),
            }
        }
        Geom2D::Arc {
            c,
            r,
            start_angle,
            end_angle,
            ccw,
        } => {
            if (sx.abs() - sy.abs()).abs() > MIN_SCALE {
                return Err(Reason::from_code(ReasonCode::EditTransformWouldDegenerate));
            }
            Geom2D::Arc {
                c: scale_point(c, center, sx, sy),
                r: *r * sx.abs(),
                start_angle: *start_angle,
                end_angle: *end_angle,
                ccw: *ccw,
            }
        }
        Geom2D::Polyline { pts, closed } => Geom2D::Polyline {
            pts: pts.iter().map(|p| scale_point(p, center, sx, sy)).collect(),
            closed: *closed,
        },
    })
}
