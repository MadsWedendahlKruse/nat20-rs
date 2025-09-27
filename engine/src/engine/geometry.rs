use parry3d::{na::Point3, shape::TriMesh};

pub struct WorldGeometry {
    pub mesh: TriMesh,
}

impl WorldGeometry {
    pub fn new(points: Vec<Point3<f32>>, indices: Vec<[u32; 3]>) -> Self {
        Self {
            mesh: TriMesh::new(points, indices).unwrap(),
        }
    }
}
