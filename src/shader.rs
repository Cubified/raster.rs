/**
 * shader.rs: Vertex and fragment shader routines.
 * 
 * Responsible for printing the output escape sequences to the screen.
 */

use std::collections::HashMap;

use crate::model::Model;
use crate::render;
use crate::texture::Texture;
use nalgebra::{Vector3, Vector4, Matrix2x3, Matrix3, Matrix4};

#[derive(Debug)]
pub struct Shader<'a> {
  pub width: i32,
  pub height: i32,

  model: &'a mut Model,
  uniform_light: Vector3<f32>,
  varying_uv: Matrix2x3<f32>,
  varying_normal: Matrix3<f32>,
  view_triangle: Matrix3<f32>,

  pub model_view: Matrix4<f32>,
  pub projection: Matrix4<f32>,
  pub viewport: Matrix4<f32>,

  diffuse_map: Texture,
  /*normal_map: Texture,
  specular_map: Texture,*/
}

impl Shader<'_> {
  pub fn new(w: usize, h: usize, m: &mut Model) -> Shader {
    Shader {
      width: w as i32,
      height: h as i32,

      model: m,
      uniform_light: Vector3::identity(),
      varying_uv: Matrix2x3::identity(),
      varying_normal: Matrix3::identity(),
      view_triangle: Matrix3::identity(),

      model_view: Matrix4::identity(),
      projection: Matrix4::identity(),
      viewport: Matrix4::identity(),

      diffuse_map: Texture::new(),
      /*normal_map: Texture::new(),
      specular_map: Texture::new(),*/
    }
  }

  pub fn vertex(&mut self, iface: usize, nthvert: usize) -> Vector4<f32> {
    if let Some(x) = self.model.uv(iface, nthvert) {
      self.varying_uv.set_column(nthvert, &x);
    }

    if let Some(x) = self.model.normal(iface, nthvert) {
      let prod = self.model_view.try_inverse().unwrap().transpose() * Vector4::new(x.x, x.y, x.z, 0.0);
      self.varying_normal.set_column(nthvert, &prod.xyz());
    }

    let out = self.model_view * self.model.vert(iface, nthvert);
    self.view_triangle.set_column(nthvert, &out.xyz());
    self.projection * out
  }

  // TODO: Overhaul this with more accurate shading.
  pub fn fragment(&self, vec: Vector3<f32>) -> Vector4<f32> {
    let bn = (self.varying_normal * vec).normalize();
    let mut uv = self.varying_uv * vec;
    uv.y = 1.0 - uv.y;

    if self.diffuse_map.loaded {
      let mut tmp = self.diffuse_map.get(uv);
      tmp /= 255.0;
      tmp.w = 1.0;
      return tmp;
    }

    let ai = Matrix3::from_columns(&[
      self.view_triangle.column(1) - self.view_triangle.column(0),
      self.view_triangle.column(2) - self.view_triangle.column(0),
      bn
    ]).try_inverse().unwrap();
    let i = ai * Vector3::new(self.varying_uv.m12 - self.varying_uv.m11, self.varying_uv.m13 - self.varying_uv.m11, 0.0);
    let j = ai * Vector3::new(self.varying_uv.m22 - self.varying_uv.m21, self.varying_uv.m23 - self.varying_uv.m21, 0.0);
    let b = Matrix3::from_columns(&[
      i.normalize(),
      j.normalize(),
      bn
    ]).transpose();

    /*if self.diffuse_map.loaded {
      let normal_raw = self.normal_map.get(uv).xyz();
      let normal_flip = Vector3::new(normal_raw.z, normal_raw.y, normal_raw.x) * 127.0;
      let normal_sub = normal_flip - Vector3::new(1.0, 1.0, 1.0);

      let n = match self.normal_map.loaded {
        true => (b * normal_sub).normalize(),
        false => (b * Vector3::new(0.0, 0.0, -1.0)).normalize(),
      };
      let diff = n.dot(&self.uniform_light).max(0.0);
      let r = ((n * diff * 2.0) - self.uniform_light).normalize();
      let spec = (-r.z).max(0.0).powf(5.0 + self.specular_map.get(uv).z);

      let diffuse = self.diffuse_map.get(uv).scale(diff + spec).add_scalar(10.0);
      Vector4::new(
        diffuse.x.clamp(0.0, 255.0) / 255.0,
        diffuse.y.clamp(0.0, 255.0) / 255.0,
        diffuse.z.clamp(0.0, 255.0) / 255.0,
        1.0
      )
    } else {*/
      let n = (b * Vector3::new(1.0, 0.0, 0.0)).normalize();
      let dot = n.dot(&self.uniform_light);
      let diff = dot.max(0.0);
      let r = ((n * dot * 2.0) - self.uniform_light).normalize();
      let spec = (-r.z).max(0.0).powf(5.0);
      let val = (diff + spec).clamp(0.0, 1.0).powf(3.0);
      Vector4::new(val, val, val, 1.0)
    // }
  }

  pub fn set_light(&mut self, light: Vector3<f32>) {
    let n = (self.model_view * Vector4::new(light.x, light.y, light.z, 0.0)).normalize();
    self.uniform_light = Vector3::new(n.x, n.y, n.z);
  }

  pub fn set_projection(&mut self, f: f32) {
    self.projection = Matrix4::new(
      1.0, 0.0, 0.0, 0.0,
      0.0, -1.0, 0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      0.0, 0.0, -1.0 / f, 0.0,
    );
  }

  pub fn look_at(&mut self, eye: &Vector3<f32>) {
    let z = (-eye).normalize();
    let x = Vector3::y().cross(&z).normalize();
    let y = z.cross(&x).normalize();

    let m_inv = Matrix4::new(
      x.x, x.y, x.z, 0.0,
      y.x, y.y, y.z, 0.0,
      z.x, z.y, z.z, 0.0,
      0.0, 0.0, 0.0, 1.0,
    );
    let tr = Matrix4::new(
      1.0, 0.0, 0.0, -eye.x,
      0.0, 1.0, 0.0, -eye.y,
      0.0, 0.0, 1.0, -eye.z,
      0.0, 0.0, 0.0, 1.0,
    );

    self.model_view = m_inv * tr;
  }

  pub fn set_modelview(&mut self, m: &Matrix4<f32>) {
    self.model_view.copy_from(m);
  }

  pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32) {
    self.viewport = Matrix4::new(
      w / 2.0, 0.0, 0.0, x + w / 2.0,
      0.0, h / 2.0, 0.0, y + h / 2.0,
      0.0, 0.0, 1.0, 0.0,
      0.0, 0.0, 0.0, 1.0,
    );
  }

  pub fn render(&mut self) {
    let mut out: HashMap<i32, String> = HashMap::with_capacity((self.width * self.height) as usize);
    let mut zbuffer: HashMap<i32, f32> = HashMap::with_capacity((self.width * self.height) as usize);
    for i in 0..self.model.nfaces() {
      let mut clip_vert = vec![];
      for j in 0..3 {
        clip_vert.push(self.vertex(i, j));
      }
      render::triangle(self, clip_vert, &mut zbuffer, &mut out);
    }

    let mut esc = String::from("\x1b[0m\x1b[0H");
    let mut prev = String::new();
    let default = String::from("\x1b[0m ");
    for y in 1..self.height {
      for x in 1..self.width {
        let cell = out.get(&(x + (y * self.width))).unwrap_or(&default);

        if *cell == prev {
          esc += " ";
        } else {
          esc += cell;
          prev.clone_from(cell);
        }
      }
      esc += "\n";
    }
    println!("{}", esc);
  }

  pub fn set_diffuse(&mut self, filename: &String) {
    self.diffuse_map.load(filename);
  }
  /*
  pub fn set_normal(&mut self, filename: &String) {
    self.normal_map.load(filename);
  }
  pub fn set_specular(&mut self, filename: &String) {
    self.specular_map.load(filename);
  }
  */
}

pub fn start() {
  println!("\x1b[?1049h\x1b[0m\x1b[2J\x1b[?1003h\x1b[?1015h\x1b[?1006h\x1b[?25l");
}
pub fn stop() {
  println!("\x1b[0m\x1b[2J\x1b[?1049l\x1b[?1003l\x1b[?1015l\x1b[?1006l\x1b[?25h");
}