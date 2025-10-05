use glam::{UVec3, Vec3A};
use obj::Obj;
use parry3d::na::Point3;
use rerecast::{
    AreaType, BuildContoursFlags, ConfigBuilder, DetailNavmesh, HeightfieldBuilder, PolygonNavmesh,
};

pub struct WorldGeometry {
    pub trimesh: parry3d::shape::TriMesh,
    pub poly_navmesh: PolygonNavmesh,
    pub detail_navmesh: DetailNavmesh,
    pub polyanya_mesh: polyanya::Mesh,
}

impl WorldGeometry {
    pub fn new(points: Vec<[f32; 3]>, indices: Vec<[u32; 3]>) -> Self {
        // See: https://github.com/janhohenheim/rerecast/blob/main/crates/rerecast/tests/cpp_comparison.rs
        // For params see: https://github.com/janhohenheim/rerecast/blob/main/crates/rerecast/src/config.rs
        // see also: https://www.youtube.com/watch?v=wYRrvWaLjJ8

        let mut nav_trimesh = rerecast::TriMesh {
            vertices: points
                .iter()
                .map(|p| Vec3A::new(p[0], p[1], p[2]))
                .collect(),
            indices: indices
                .iter()
                .map(|i| UVec3::new(i[0], i[1], i[2]))
                .collect(),
            area_types: vec![AreaType::DEFAULT_WALKABLE; indices.len()],
        };

        nav_trimesh.mark_walkable_triangles(f32::to_radians(45.0));

        let aabb = nav_trimesh.compute_aabb().unwrap();

        let cell_size = 0.1;
        let cell_height = 0.15;

        let mut heightfield = HeightfieldBuilder {
            aabb,
            cell_size,
            cell_height,
        }
        .build()
        .unwrap();

        let walkable_height = (1.8 / cell_height).ceil() as u16;
        let walkable_climb = (0.6 / cell_height).ceil() as u16;
        let walkable_radius = (0.6 / cell_size).ceil() as u16;

        heightfield
            .rasterize_triangles(&nav_trimesh, walkable_climb)
            .unwrap();

        // Once all geometry is rasterized, we do initial pass of filtering to
        // remove unwanted overhangs caused by the conservative rasterization
        // as well as filter spans where the character cannot possibly stand.
        heightfield.filter_low_hanging_walkable_obstacles(walkable_climb);
        heightfield.filter_ledge_spans(walkable_height, walkable_climb);
        heightfield.filter_walkable_low_height_spans(walkable_height);

        let mut compact_heightfield = heightfield
            .into_compact(walkable_height, walkable_climb)
            .unwrap();

        compact_heightfield.erode_walkable_area(walkable_radius);

        // TODO: What are these convex volumes? Where do they come from?

        // let volumes = load_json::<CppVolumes>(project, "convex_volumes");
        // for volume in volumes.volumes {
        //     let volume = ConvexVolume {
        //         vertices: volume
        //             .verts
        //             .iter()
        //             .map(|[x, _y, z]| Vec2::new(*x, *z))
        //             .collect(),
        //         min_y: volume.hmin,
        //         max_y: volume.hmax,
        //         area: AreaType::from(volume.area),
        //     };
        //     compact_heightfield.mark_convex_poly_area(&volume);
        // }

        compact_heightfield.build_distance_field();

        let border_size = 0;
        let min_region_area = 8;
        let merge_region_area = 20;

        compact_heightfield
            .build_regions(border_size, min_region_area, merge_region_area)
            .unwrap();

        let max_simplification_error = 1.3;
        let max_edge_len = walkable_radius * 8;

        let contours = compact_heightfield.build_contours(
            max_simplification_error,
            max_edge_len,
            BuildContoursFlags::DEFAULT,
        );

        let max_vertices_per_polygon = 3;

        let poly_navmesh = contours
            .into_polygon_mesh(max_vertices_per_polygon)
            .unwrap();

        // TEMP
        let snapped_nav_aabb = poly_navmesh.aabb;
        let dx = snapped_nav_aabb.min.x - aabb.min.x;
        let dy = snapped_nav_aabb.min.y - aabb.min.y;
        let dz = snapped_nav_aabb.min.z - aabb.min.z;
        println!("grid snap delta = ({dx:.6}, {dy:.6}, {dz:.6})");
        println!(
            "cell_size = {}, cell_height = {}",
            poly_navmesh.cell_size, poly_navmesh.cell_height
        );
        // ---

        let detail_sample_dist = 6.0;
        let detail_sample_max_error = 1.0;

        let detail_navmesh = DetailNavmesh::new(
            &poly_navmesh,
            &compact_heightfield,
            detail_sample_dist,
            detail_sample_max_error,
        )
        .unwrap();

        let polyanya_mesh =
            polyanya::RecastFullMesh::new(poly_navmesh.clone(), detail_navmesh.clone()).into();

        Self {
            trimesh: parry3d::shape::TriMesh::new(
                points.iter().map(|p| Point3::from_slice(p)).collect(),
                indices.clone(),
            )
            .unwrap(),
            poly_navmesh,
            detail_navmesh,
            polyanya_mesh,
        }
    }
}

impl From<Obj> for WorldGeometry {
    fn from(obj: Obj) -> Self {
        let points = obj
            .vertices
            .iter()
            .map(|v| [v.position[0], v.position[1], v.position[2]])
            .collect();

        let indices = obj
            .indices
            .chunks(3)
            .map(|chunk| [chunk[0] as u32, chunk[1] as u32, chunk[2] as u32])
            .collect();

        Self::new(points, indices)
    }
}
