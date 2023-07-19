use nalgebra::{Point3, Vector3};

struct Cuboid {
    pos:Point3<f32>,
    size:Vector3<f32>
}

enum CollisionResult {
    NoCollision,
    Collision,
    Contained,
    Contains
}

