use spirv_std::vector::Vector;

unsafe impl Vector<f32, 2> for crate::Vector2 {}
unsafe impl Vector<f32, 3> for crate::Vector3 {}
unsafe impl Vector<f32, 4> for crate::Vector4 {}