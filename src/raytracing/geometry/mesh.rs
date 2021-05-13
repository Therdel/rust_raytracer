use crate::raytracing::Triangle;

pub struct Mesh<'a> {
    pub id: String,
    pub triangles: Vec<Triangle<'a>>
}