use rapier3d::na::{Const, OPoint};

pub fn get_vertices(file_path: &str) -> (Vec<OPoint<f32, Const<3>>>, Vec<[u32; 3]>) {
    let bytes = std::fs::read(file_path).unwrap();
    let file_str = String::from_utf8(bytes).unwrap();
    let mut vertices = Vec::<OPoint<f32, Const<3>>>::new();
    let mut indices = Vec::<[u32; 3]>::new();
    for line in file_str.lines().into_iter() {
        let tokens : Vec<&str> = line.split_whitespace().collect();
        match tokens[0] {
            "v" => {
                let coordinates = [tokens[1].parse::<f32>().unwrap(), tokens[2].parse::<f32>().unwrap(), tokens[3].parse::<f32>().unwrap()];
                vertices.push(OPoint::from_slice(&coordinates));
            },
            "f" => {
                let indexes = [split(tokens[1]), split(tokens[2]), split(tokens[3])];
                indices.push(indexes)
            },
            _ => continue
        }
    }
    println!("{:?}\n{:?}", vertices, indices);
    (vertices, indices)
}

fn split(data: &str) -> u32 {
    data.split("/").collect::<Vec<&str>>()[0].parse::<u32>().unwrap() - 1
}