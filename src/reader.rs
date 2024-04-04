use obj::*;
use rapier3d::na::{Const, OPoint};

pub fn get_vertices(model_path: &str) -> Vec<OPoint<f32, Const<3>>> {
    let buf = std::io::BufReader::new(std::fs::File::open(model_path).unwrap());
    let object_data: Obj = load_obj(buf).unwrap();
    let vertices = object_data.vertices;
    let mut v = Vec::<OPoint<f32, Const<3>>>::new();
    for vertice in vertices {
        v.push(OPoint::from_slice(&vertice.position));
    }
    v
}
