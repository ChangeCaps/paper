use glam::*;

#[derive(Clone, Debug)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    #[inline]
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Self::IDENTITY
        }
    }

    #[inline]
    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Self::IDENTITY
        }
    }

    #[inline]
    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Self::IDENTITY
        }
    }

    #[inline]
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    #[inline]
    pub fn mul_vec3(&self, mut vec3: Vec3) -> Vec3 {
        vec3 *= self.scale;
        vec3 = self.rotation * vec3;
        vec3 + self.translation
    }

    #[inline]
    pub fn mul_transform(&self, other: Self) -> Self {
        Self {
            translation: self.mul_vec3(other.translation),
            rotation: self.rotation * other.rotation,
            scale: self.scale * other.scale,
        }
    }
}

impl Default for Transform {
    #[inline]
    fn default() -> Self {
        Transform::IDENTITY
    }
}

impl Into<Mat4> for Transform {
    #[inline]
    fn into(self) -> Mat4 {
        self.matrix()
    }
}
