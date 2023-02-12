use glam::{Mat4, Quat, Vec2, Vec3, Vec4};

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub colour: Vec3,
    pub uv: Vec2,
}

#[derive(Debug, Copy, Clone)]
pub struct FragIn {
    pub position: Vec4,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub colour: Vec3,
    pub uv: Vec2,
}

pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl FragIn {
    pub fn lerp(&self, rhs: FragIn, t: f32) -> FragIn {
        FragIn {
            position: self.position.lerp(rhs.position, t),
            normal: self.normal.lerp(rhs.normal, t),
            tangent: self.tangent.lerp(rhs.tangent, t),
            colour: self.colour.lerp(rhs.colour, t),
            uv: self.uv.lerp(rhs.uv, t),
        }
    }
}

impl Transform {
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    pub fn forward(&self) -> Vec3 {
        self.rotation * -Vec3::Z
    }
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(
            self.translation,
            self.translation + self.forward(),
            glam::vec3(0.0, 1.0, 0.0),
        )
    }
    pub fn trans_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
    pub fn local_matrix(&self) -> Mat4 {
        Mat4::from_translation(self.translation)
            * Mat4::from_quat(self.rotation)
            * Mat4::from_scale(self.scale)
    }
}
