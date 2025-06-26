#[derive(Clone, Debug, Copy)]
pub struct Plane {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
}

impl Plane {
    pub fn new(a_f32: f32, b_f32: f32, c_f32: f32, d_f32: f32) -> Plane {
        Plane {
            a: a_f32,
            b: b_f32,
            c: c_f32,
            d: d_f32,
        }
    }
}
