/**
 * render.rs: Triangle rasterization routines.
 */

use crate::shader::Shader;
use std::cmp::{min, max};
use nalgebra::{Vector2, Vector3, Vector4, Matrix2x3, Matrix3};

#[derive(Debug)]
pub struct Vertex {
  pub clip: [Vector4<f32>; 3],
  pub varying_uv: Matrix2x3<f32>,
  pub varying_normal: Matrix3<f32>,
  pub view_triangle: Matrix3<f32>,
}

impl Vertex {
  pub fn new() -> Vertex {
    Vertex {
      clip: [Vector4::zeros(); 3],
      varying_uv: Matrix2x3::identity(),
      varying_normal: Matrix3::identity(),
      view_triangle: Matrix3::identity(),
    }
  }

  pub fn triangle(
    &self,
    s: &Shader
  ) -> Vec<(i32, f32, String)> {
    let pts1 = [
      s.viewport * self.clip[0],
      s.viewport * self.clip[1],
      s.viewport * self.clip[2],
    ];
    let pts2 = vec![
      Vector2::new(pts1[0].x / pts1[0].w, pts1[0].y / pts1[0].w),
      Vector2::new(pts1[1].x / pts1[1].w, pts1[1].y / pts1[1].w),
      Vector2::new(pts1[2].x / pts1[2].w, pts1[2].y / pts1[2].w),
    ];

    let mut bboxmin = Vector2::new(s.width, s.height);
    let mut bboxmax = Vector2::new(1, 1);
    for vec in pts2.iter().take(3) {
      bboxmin.x = min(bboxmin.x, vec.x as i32);
      bboxmin.y = min(bboxmin.y, vec.y as i32);

      bboxmax.x = max(bboxmax.x, vec.x as i32);
      bboxmax.y = max(bboxmax.y, vec.y as i32);
    }

    let xmin = max(bboxmin.x, 1);
    let xmax = min(bboxmax.x, s.width) + 2;

    let ymin = max(bboxmin.y, 1);
    let ymax = min(bboxmax.y, s.height) + 2;

    let mut out = vec![];
    for x in xmin..xmax {
      for y in ymin..ymax {
        let p = Vector2::new(x as f32, y as f32);
        let bc_screen = barycentric(&pts2, p);
        if bc_screen.x < 0.0 || bc_screen.y < 0.0 || bc_screen.z < 0.0 {
          continue;
        }

        let mut bc_clip = Vector4::new(
          bc_screen.x / pts1[0].w,
          bc_screen.y / pts1[1].w,
          bc_screen.z / pts1[2].w,
          1.0,
        );
        bc_clip /= bc_clip.x + bc_clip.y + bc_clip.z;

        let frag_depth = Vector4::new(self.clip[0].z, self.clip[1].z, self.clip[2].z, 0.0).dot(&bc_clip);
        let z_idx = x + (s.height - y) * s.width;

        let v = s.fragment(self, (s.model_view * bc_clip).xyz(), bc_clip.xyz());
        if v.w == 0.0 {
          continue;
        }

        let seq = format!(
          "\x1b[48;2;{};{};{}m ",
          (v.w * v.x) as i32,
          (v.w * v.y) as i32,
          (v.w * v.z) as i32
        );
        out.push((
          z_idx,
          frag_depth,
          seq
        ));
      }
    }
    out
  }
}

fn barycentric(pts: &[Vector2<f32>], p: Vector2<f32>) -> Vector3<f32> {
  let abc = Matrix3::new(
    pts[0].x, pts[0].y, 1.0,
    pts[1].x, pts[1].y, 1.0,
    pts[2].x, pts[2].y, 1.0,
  );
  if abc.determinant() < 1e-3 {
    return Vector3::new(-1.0, 1.0, 1.0);
  }

  abc.try_inverse().unwrap().transpose() * Vector3::new(p.x, p.y, 1.0)
}
