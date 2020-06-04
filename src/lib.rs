use ggez::nalgebra::Point2;
use mun_runtime::StructRef;

pub fn marshal_vec2(pos: &StructRef) -> Point2<f32> {
    Point2::new(pos.get("x").unwrap(), pos.get("y").unwrap())
}
