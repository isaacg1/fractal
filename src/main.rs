#![feature(array_zip)]
use image::{ImageError, Rgb, RgbImage};
use rand::prelude::*;
use rand_distr::{Normal, Triangular};

use std::f64::consts::PI;

type Pixel = [usize; 2];
type Color = [f64; 3];

#[derive(Debug)]
struct Transformation {
    src_center: Pixel,
    src_radius: usize,
    dst_center: Pixel,
    dst_radius: usize,
    color_offset: Color,
    rotation: f64,
}

impl Transformation {
    fn generate<R: Rng>(size: usize, rng: &mut R) -> Transformation {
        let src_center = [size; 2].map(|s| rng.gen_range(0..s));
        let clearance = src_center[0]
            .min(src_center[1])
            .min(size - 1 - src_center[0])
            .min(size - 1 - src_center[1]);
        let src_radius = rng.gen_range(0..=clearance);
        let dst_radius = rng.gen_range(0..=src_radius);
        let dst_center = [0; 2].map(|_| rng.gen_range(dst_radius..size - dst_radius));
        // i.i.d. normal is circularly symmetric
        let normal = Normal::new(0.0, 1.0).unwrap();
        let color_offset = [0; 3].map(|_| normal.sample(rng));
        let rotation = rng.gen_range(0.0..2.0 * PI);
        Transformation {
            src_center,
            src_radius,
            dst_center,
            dst_radius,
            color_offset,
            rotation,
        }
    }
}

fn make_image(unique_trans: usize, num_trans: usize, size: usize, seed: u64) -> RgbImage {
    let mut rng = StdRng::seed_from_u64(seed);
    let transes: Vec<Transformation> = (0..unique_trans)
        .map(|_| Transformation::generate(size, &mut rng))
        .collect();
    let mut img: Vec<Vec<Color>> = vec![vec![[0.0; 3]; size]; size];
    let triangle = Triangular::new(0., unique_trans as f64, 0.).unwrap();
    for _ in 0..num_trans {
        let index = triangle.sample(&mut rng) as usize;
        let trans = &transes[index];
        // If src and dst overlap, strange effects?
        for dr in -(trans.src_radius as isize)..trans.src_radius as isize {
            for dc in -(trans.src_radius as isize)..trans.src_radius as isize {
                let src_dist_sq = dr.pow(2) + dc.pow(2);
                if src_dist_sq as usize > trans.src_radius.pow(2) {
                    continue;
                }
                let src_dist = (src_dist_sq as f64).sqrt();
                let dst_dist = src_dist * trans.dst_radius as f64 / trans.src_radius as f64;
                let src_angle = (dc as f64).atan2(dr as f64);
                let dst_angle = src_angle + trans.rotation;
                let src_r = (trans.src_center[0] as isize + dr) as usize;
                let src_c = (trans.src_center[1] as isize + dc) as usize;
                let src_color = img[src_r][src_c];
                let dst_color = src_color.zip(trans.color_offset).map(|(f1, f2)| f1 + f2);
                let dst_r = (trans.dst_center[0] as f64 + dst_angle.cos() * dst_dist) as usize;
                let dst_c = (trans.dst_center[1] as f64 + dst_angle.sin() * dst_dist) as usize;
                img[dst_r][dst_c] = dst_color;
            }
        }
    }
    let mut buckets = vec![];
    let bucket_width = 0.01;
    for row in &img {
        for color in row {
            for channel in color {
                let index = (channel / bucket_width).abs() as usize;
                if buckets.len() <= index {
                    buckets.extend((0..(index - buckets.len() + 1)).map(|_| 0));
                }
                buckets[index] += 1;
            }
        }
    }
    let mut count = 0;
    let mut characteristic_width = bucket_width;
    for (i, b) in buckets.iter().enumerate() {
        if i == 0 {
            count += b/2
        } else {
            count += b;
        }
        if count > size.pow(2) * 3 / 2 {
            characteristic_width = i as f64 * bucket_width;
            break;
        }
    }
    println!("{} {}", count, characteristic_width);
    let mut out_img = RgbImage::new(size as u32, size as u32);
    for (r, row) in img.iter().enumerate() {
        for (c, color) in row.iter().enumerate() {
            let fixed_color =
                color.map(|f| (255.0 / (1.0 + (f / characteristic_width).exp())).round() as u8);
            out_img.put_pixel(r as u32, c as u32, Rgb(fixed_color));
        }
    }
    out_img
}

fn main() -> Result<(), ImageError> {
    let unique_trans = 100;
    let num_trans = 1000;
    let size = 1000;
    let seed = 0;
    let filename = format!("img-{}-{}-{}-{}.png", unique_trans, num_trans, size, seed);
    println!("{}", filename);
    let img = make_image(unique_trans, num_trans, size, seed);
    img.save(filename)
}
