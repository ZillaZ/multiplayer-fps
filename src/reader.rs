use self::objects::Object;
use crate::*;
use rapier3d::{
    dynamics::RigidBodyType,
    na::{Const, OPoint},
};

fn move_to_origin(vertices: &mut Vec<OPoint<f32, Const<3>>>) -> (Vector3, f32) {
    let mins = get_max_axis(vertices);
    let (min_x, max_x) = (mins[0].0, mins[0].1);
    let (min_y, max_y) = (mins[1].0, mins[1].1);
    let (min_z, max_z) = (mins[2].0, mins[2].1);

    let radius = (max_x - min_x) / 2.0;
    let diff_x = (max_x + min_x) / 2.0;
    let diff_y = (max_y + min_y) / 2.0;
    let diff_z = (max_z + min_z) / 2.0;
    let obj_position = Vector3::new(diff_x, diff_y, diff_z);
    for vertice in vertices.iter_mut() {
        vertice[0] -= diff_x;
        vertice[1] -= diff_y;
        vertice[2] -= diff_z;
    }
    (obj_position, radius)
}

fn get_max_axis(vertices: &mut Vec<OPoint<f32, Const<3>>>) -> Vec<(f32, f32)> {
    let mut max_x = vertices.iter().map(|x| x[0]).collect::<Vec<f32>>();
    max_x.sort_by(|a, b| a.total_cmp(b));
    let mut max_y = vertices.iter().map(|x| x[1]).collect::<Vec<f32>>();
    max_y.sort_by(|a, b| a.total_cmp(b));
    let mut max_z = vertices.iter().map(|x| x[2]).collect::<Vec<f32>>();
    max_z.sort_by(|a, b| a.total_cmp(b));
    vec![
        (max_x[0], max_x.pop().unwrap()),
        (max_y[0], max_y.pop().unwrap()),
        (max_z[0], max_z.pop().unwrap()),
    ]
}

pub fn load_scene(file_path: &str) -> Vec<Object> {
    let bytes = std::fs::read(file_path).unwrap();
    let file_str = String::from_utf8(bytes).unwrap();

    let mut vertices = Vec::<OPoint<f32, Const<3>>>::new();
    let mut indices = Vec::<[u32; 3]>::new();

    let mut vertice_count = 0;
    let mut last_vertice_count = 0;

    let mut obj_info = "";
    let mut obj_rotation = Vector3::zero();
    let mut body_type: Option<RigidBodyType> = None;
    let mut object_shape: Option<S> = None;

    let mut objects = Vec::<Object>::new();

    for line in file_str.lines().into_iter() {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        match tokens[0] {
            "v" => {
                let coordinates = [
                    tokens[1].parse::<f32>().unwrap(),
                    tokens[2].parse::<f32>().unwrap(),
                    tokens[3].parse::<f32>().unwrap(),
                ];
                vertices.push(OPoint::from_slice(&coordinates));
                vertice_count += 1;
            }
            "f" => {
                let indexes = [
                    split(tokens[1]) - last_vertice_count,
                    split(tokens[2]) - last_vertice_count,
                    split(tokens[3]) - last_vertice_count,
                ];
                indices.push(indexes);
            }
            "o" => {
                if object_shape.is_some() {
                    let (obj_position, radius) = move_to_origin(&mut vertices);

                    last_vertice_count = vertice_count;
                    objects.push(Object::new(
                        object_shape.clone().unwrap(),
                        body_type.unwrap(),
                        vertices.clone(),
                        indices.clone(),
                        obj_rotation,
                        obj_position,
                        radius,
                        obj_info.into(),
                    ));
                    vertices.clear();
                    indices.clear();
                }
                obj_info = tokens[1].split("-").collect::<Vec<&str>>()[0];
                let aux = tokens[1].split("-").collect::<Vec<&str>>()[1];
                let (x, y, z) = (
                    aux.split(",").collect::<Vec<&str>>()[0]
                        .parse::<f32>()
                        .unwrap(),
                    aux.split(",").collect::<Vec<&str>>()[1]
                        .parse::<f32>()
                        .unwrap(),
                    aux.split(",").collect::<Vec<&str>>()[2]
                        .parse::<f32>()
                        .unwrap(),
                );
                obj_rotation = Vector3::new(y.to_radians(), z.to_radians(), x.to_radians());
                body_type = match &obj_info[0..1] {
                    "D" => Some(RigidBodyType::Dynamic),
                    "F" => Some(RigidBodyType::Fixed),
                    _ => panic!("wtf is that body type bruh."),
                };
                object_shape = match &obj_info[1..2] {
                    "C" => Some(S::CONVEX),
                    "M" => Some(S::MULTI),
                    "S" => Some(S::SensorMulti),
                    "B" => Some(S::SPHERE(Sphere::new(1.0))),
                    _ => panic!("bruhhh."),
                };
            }
            _ => continue,
        }
    }
    let (obj_position, radius) = move_to_origin(&mut vertices);

    objects.push(Object::new(
        object_shape.clone().unwrap(),
        body_type.unwrap(),
        vertices.clone(),
        indices.clone(),
        obj_rotation,
        obj_position,
        radius,
        obj_info.into(),
    ));
    vertices.clear();
    indices.clear();
    objects
}

fn split(data: &str) -> u32 {
    data.split("/").collect::<Vec<&str>>()[0]
        .parse::<u32>()
        .unwrap()
        - 1
}
