use crate::AiReal;

#[derive(Clone, Debug, Copy)]
pub struct Plane {
    pub a: AiReal,
    pub b: AiReal,
    pub c: AiReal,
    pub d: AiReal,
}

impl Plane {
    pub const fn new(a: AiReal, b: AiReal, c: AiReal, d: AiReal) -> Plane {
        Plane { a, b, c, d }
    }
}
