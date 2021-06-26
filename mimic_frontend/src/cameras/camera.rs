use glm::Vec3;

pub struct Camera {
    pub position: Vec3,
    pub forward: Vec3,
}

impl Camera {
    pub fn new(position: Vec3, forward: Vec3) -> Self {
        Self { position, forward }
    }
}
