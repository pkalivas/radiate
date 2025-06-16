mod gene;
mod polygon;

// use radiate::*;

fn main() {
    let image_file_path = std::env::current_dir()
        .unwrap()
        .join("examples/image-evo/monalisa.png");

    let image = image::open(image_file_path).unwrap();
    let image = image.to_rgba8();
    let _ = image.pixels().collect::<Vec<_>>();
}
