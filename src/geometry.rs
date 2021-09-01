use na::Vector3;
use nalgebra as na;

/// Finds the intersection of the given line and box, both centered at origin.
/// The order of the two intersections are undefined.
/// Providing `NaN` values will cause a panic.
pub fn intersect_box_and_line(line_dir: Vector3<f32>, box_size: Vector3<f32>) -> [Vector3<f32>; 2] {
    let line_dir = line_dir.normalize();

    let plane_centers = [
        Vector3::new(box_size.x / 2.0, 0.0, 0.0),
        Vector3::new(-box_size.x / 2.0, 0.0, 0.0),
        Vector3::new(0.0, box_size.y / 2.0, 0.0),
        Vector3::new(0.0, -box_size.y / 2.0, 0.0),
        Vector3::new(0.0, 0.0, box_size.z / 2.0),
        Vector3::new(0.0, 0.0, -box_size.z / 2.0),
    ];

    let mut intersections = Vec::with_capacity(6);

    for plane in plane_centers {
        let intersection = intersect_plane_and_line(line_dir, plane, plane.normalize());
        intersections.push(intersection);
    }

    intersections = intersections
        .into_iter()
        .filter(|&vec| vec.norm().is_finite())
        .collect();
    intersections.sort_unstable_by(|&a, &b| a.norm().partial_cmp(&b.norm()).unwrap());

    [intersections[0], intersections[1]]
}

/// Written with reference to
/// https://rosettacode.org/wiki/Find_the_intersection_of_a_line_with_a_plane#Rust
pub fn intersect_plane_and_line(
    ray_vector: Vector3<f32>,
    plane_point: Vector3<f32>,
    plane_normal: Vector3<f32>,
) -> Vector3<f32> {
    let ray_point = Vector3::zeros();
    let diff = ray_point - plane_point;
    let prod1 = diff.dot(&plane_normal);
    let prod2 = ray_vector.dot(&plane_normal);
    let prod3 = prod1 / prod2;
    ray_point - ray_vector.scale(prod3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use na::Vector3;
    use nalgebra as na;

    #[test]
    fn intersect_box_and_line_xyz() {
        let line_dir = Vector3::new(1.0, 1.0, 1.0);
        let box_size = Vector3::new(2.0, 2.0, 2.0);
        let result = intersect_box_and_line(line_dir, box_size);

        // Result order is undefined, so check either order.
        let expect1 = [Vector3::new(1.0, 1.0, 1.0), Vector3::new(-1.0, -1.0, -1.0)];
        let expect2 = [Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0)];
        assert!(result == expect1 || result == expect2);
    }

    #[test]
    fn intersect_plane_and_line_z_only() {
        let ray_vector = Vector3::new(0.0, 0.0, 1.0);
        let plane_point = Vector3::new(0.0, 0.0, 1.0);
        let plane_normal = Vector3::new(0.0, 0.0, 1.0);
        let result = intersect_plane_and_line(ray_vector, plane_point, plane_normal);
        let expect = Vector3::new(0.0, 0.0, 1.0);
        assert_eq!(result, expect);
    }
}
