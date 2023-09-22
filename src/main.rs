/**
 * main.rs: Program entry point.
 */

mod model;
mod render;
mod shader;
mod texture;
mod orbit;
mod esc;
mod ui;

use std::env;

use nalgebra::Vector3;

fn main() {
  let (w, h) = match term_size::dimensions() {
    Some((w, h)) => (w, h),
    None => panic!("Unable to get terminal size"),
  };
  let width = w as f32;
  let height = h as f32;

  let args: Vec<String> = env::args().collect();
  if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
    // eprintln!("Usage: raster [model.obj] {{diffuse.tga}} {{normal.tga}} {{specular.tga}}");
    eprintln!("Usage: raster [model.obj] {{diffuse.tga}}");
    return;
  }

  let mut obj = model::Model::load_obj(&args[1]).unwrap();
  let mut shader = shader::Shader::new(w, h, &mut obj);

  shader.set_light(Vector3::new(-3.0, 0.0, 0.0));
  shader.set_viewport(width / 8.0, height / 8.0, width * 0.75, height * 0.75);
  shader.set_projection(20.0);

  if args.len() > 2 {
    shader.set_diffuse(&args[2]);
  }
  /*
  if args.len() > 3 {
    shader.set_normal(&args[3]);
  }
  if args.len() > 4 {
    shader.set_specular(&args[4]);
  }
  */

  ui::UI::new(&mut shader).run();
}