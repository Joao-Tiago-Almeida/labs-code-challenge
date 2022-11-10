use image::{Rgba, RgbaImage};
use std::{cmp::{max, min}};
use rand::Rng;

// type alias, so we can use the type ImgRGBA instead of ImageBuffer<Rgba<u8>, Vec<u8>> 
type ImgRGBA = image::ImageBuffer<Rgba<u8>, Vec<u8>>;
// Triangle is a shape that will be drawn into the image
#[derive(Clone)]
struct Triangle {
    points: [Point; 3],
    color: Rgba<u8>
}
// Point is used by the Triangle struct and represents a vertice
#[derive(Clone)]
struct Point { // TODO if we set the max length of image to 256, it is only need 1 byte to represent the coordinates <u8>
    x: u32, 
    y: u32
}

// Main is the entry point of the program
fn main() {
    let begin = Instant::now();
    
    let target_image_path = "target.png";
    let output_image_path = "output.png";

    // creates a blank image we're going to paint in
    let mut image = RgbaImage::new(128, 128);
    
    // opens a reference image for the fitness func
    let ref_image = image::open(target_image_path).unwrap().into_rgba8();

    // creates the triangles
    let mut shapes: Vec<Triangle> = vec![];
    for _ in 0..50 {
        let triangle = random_triangle(image.width() as i32);
        shapes.push(triangle);
    }

    // init the distance (output for the fitness fn)
    let mut best_distance = 100f64;
    
    // NEW: get the starting time
    use std::time::Instant;
    let mut duration = 0;
    let mut now;
    let epochs = 5000;

    // main loop, runs mutation, gets fitness (distance between 2 images), keeps or discards a mutation
    for i in 0..epochs{
        now = Instant::now();
        // create a new image with white background
        init_image(&mut image);

        // mutate a shape and get a copy of the shapes vector
        let new_shapes = mutate(&shapes, image.width() as i32);
        
        // draw in the new image the vec of triangles with the mutated triangle
        draw(&mut image, &new_shapes);

        // get the distance between the new image and the reference image
        let distance = fitness(&image, &ref_image, image.width());

        // if the new distance is better than the best distance, we accept the mutation
        if best_distance > distance {
            shapes = new_shapes;
            best_distance = distance;
        }

        duration += now.elapsed().as_millis(); // NEW
        // println!("Mutation #{} - current distance: {}", i, best_distance);
        
    }

    println!("Computational time for {} epochs: {:.3} seconds with rate of {:.3} epoch/second", epochs, (duration as f32)/1000.0, (epochs as f32)/((duration as f32)/1000.0)); // NEW

    draw(&mut image, &shapes);
    _ = image.save(output_image_path);

    println!("Best fitness {}", best_distance);
    println!("Total running time {:.3} seconds", (begin.elapsed().as_millis() as f32)/1000.0);
}

// init_image creates a new image with a white background
fn init_image (image: &mut ImgRGBA) {
    for i in 0..image.width() {
        for j in 0..image.height() {
            image.put_pixel(i, j, Rgba([255, 255, 255, 255]));
        }
    }
}

// random_triangle creates and returns a random triangle
// with random vertices (points) and random color
fn random_triangle(w: i32) -> Triangle {
    Triangle { 
        points: [
            random_point(w),
            random_point(w),
            random_point(w),
        ],
        color: random_color_rgba() 
    }
}

// random_color_rgba creates and return a random rgba color
fn random_color_rgba() -> Rgba<u8> {
    let color= [
        rand::thread_rng().gen(),
        rand::thread_rng().gen(), 
        rand::thread_rng().gen(),
        rand::thread_rng().gen()
    ];
    Rgba(color)
}

// draw draws a vec of shapes into an the pixel buffer
fn draw(image: &mut ImgRGBA, shapes: &Vec<Triangle>) {
    for shape in shapes.iter() {
        draw_triangle(shape, image);
    }
 }

// Fitness returns the average rgb color distance between 2 images
// it basically compares all pixels for 2 given images and returns
// a percentage that represents the similarities between the 2 images
// 0: the 2 images are the same
fn fitness(image: &ImgRGBA, ref_image: &ImgRGBA, w: u32) -> f64 {
    
    let mut tot = 0f64;
    let wh = w as f64;

    for i in 0..w {
        for j in 0..w {
            let p1 = image.get_pixel(i, j);
            let p2 = ref_image.get_pixel(i, j);
            let distance = color_distance(p1, p2);            
            tot = tot + distance;
        }
    }
    return tot / (wh*wh);
}


// color_distance returns the distance between 2 RGB colors
fn color_distance(color_1: &Rgba<u8>, color_2: &Rgba<u8>) -> f64{

    let r1 = color_1[0] as f64;
    let g1 = color_1[1] as f64;
    let b1 = color_1[2] as f64;

    let r2 = color_2[0] as f64;
    let g2 = color_2[1] as f64;
    let b2 = color_2[2] as f64;

    let result = (r1 - r2)*(r1 - r2) + (g1 - g2)*(g1 - g2) + (b1 - b2)*(b1 - b2);
    result.sqrt() / 2.55
}

// random_point creates and returns a random point
fn random_point(w: i32) -> Point {
    Point{ 
        x: rand::thread_rng().gen_range(0..=w as u32),
        y: rand::thread_rng().gen_range(0..=w as u32) 
    }
}

// Mutate mutates a vertice coordinates or a color
fn mutate<'a>(shapes: &'a Vec<Triangle>, w: i32) -> Vec<Triangle> {

    let mut shapes_copy = shapes.to_vec();

    let point_mutation = rand::thread_rng().gen_range(0..2);
    let index = rand::thread_rng().gen_range(0..shapes.len());

    if point_mutation == 1 { // here we mutate a vertice
        let new_point = random_point(w);
        let vertice_index = rand::thread_rng().gen_range(0..3);
        shapes_copy[index].points[vertice_index] = new_point;
    } else { // here we mutate a color
        let new_color = random_color_rgba();
        shapes_copy[index].color = new_color;
    }

    // TODO change the stacking order to improve the accuracy
    return shapes_copy
}

// blend_color blends 2 colors together
fn blend_color(c1 :&Rgba<u8>, c2: &Rgba<u8>) -> Rgba<u8> {
    let r1 = c1[0] as f32;
    let g1 = c1[1] as f32;
    let b1 = c1[2] as f32;

    let r2 = c2[0] as f32;
    let g2 = c2[1] as f32;
    let b2 = c2[2] as f32;

    let alpha = c2[3] as f32 / 255f32;

    let new_color = Rgba(
        [
            (r1 * (1. - alpha) + r2 * alpha) as u8,
            (g1 * (1. - alpha) + g2 * alpha) as u8,
            (b1 * (1. - alpha) + b2 * alpha) as u8,
            (255) as u8
        ]
    );
    return new_color;
}

// draw_triangle draws a triangle in a given image
fn draw_triangle(triangle: &Triangle, image: &mut ImgRGBA) {
    let x1 = triangle.points[0].x as i32;
    let y1 = triangle.points[0].y as i32;

    let x2 = triangle.points[1].x as i32;
    let y2 = triangle.points[1].y as i32;

    let x3 = triangle.points[2].x as i32;
    let y3 = triangle.points[2].y as i32;

    let xmin = min(x1, min(x2, x3)) as i32;
    let xmax = min(max(x1, max(x2, x3)), image.width() as i32); // TODO this should already be clamped before
    let ymin = min(y1, min(y2, y3)) as i32;
    let ymax = min(max(y1, max(y2, y3)), image.height() as i32);

    // TODO http://totologic.blogspot.com/2014/01/accurate-point-in-triangle-test.html
    for x in xmin .. xmax  {
        for y in ymin .. ymax {
            let asx = x - x1;
            let asy = y - y1;

            let sab = (x2 - x1) * asy - (y2 - y1) * asx > 0;
            if ((x3 - x1) * asy - (y3 - y1) * asx > 0) == sab { continue };
            if ((x3 - x2) * (y - y2) - (y3 - y2) * (x - x2) > 0) != sab { continue };

            let current_pixel_color = image.get_pixel(x as u32, y as u32);
            let color = blend_color(current_pixel_color, &triangle.color);
            image.put_pixel(x as u32, y as u32, color)
        }
    }
}

// TODO look at this, to prevent compute all triangles every single time. If I get how the image was before, and the transformations afterwards, I then only need to change that specific layer
