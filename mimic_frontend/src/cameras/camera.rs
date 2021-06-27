use glm::Vec3;
//////////////////////// Structs ///////////////////////
pub struct Camera {
    pub position: Vec3,
    pub forward: Vec3,
}
//////////////////////// Impls ///////////////////////
impl Camera {
    pub fn new(position: Vec3, forward: Vec3) -> Self {
        Self { position, forward }
    }
}
