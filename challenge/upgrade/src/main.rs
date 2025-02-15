use image::{Rgba, RgbaImage};
use std::{cmp::{max, min}};
use rand::Rng;
use std::time::Instant;
use std::env;

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

    let args: Vec<String> = env::args().collect();

    let target_image_path = "./images/monalisa.png";
    let output_image_path = "./images/output-monalisa.png";

    // number of epochs
    let epochs = args[1].parse::<i32>().unwrap();

    let (width, height) = image::image_dimensions(target_image_path).unwrap();

    // creates a blank image we're going to paint in
    let mut image = RgbaImage::new(width, height);
    
    // opens a reference image for the fitness func
    let ref_image = image::open(target_image_path).unwrap().into_rgba8();

    let n_shapes = 50; // total number of shapes

    // stores the newest random modification
    let mut new_shape: Triangle = random_triangle(width, height);

    // creates the triangles
    // let mut shapes: Vec<Triangle> = vec![new_shape.clone(); n_shapes];
    let mut shapes: Vec<Triangle> = vec![];
    for _ in 0..n_shapes {
        shapes.push(random_triangle(width, height));
    }

    // init the distance (output for the fitness fn)
    let mut best_distance = u32::MAX;

    // distance between images
    let mut distance: u32 = 0;

    // NEW: get the starting time
    let mut duration = 0;
    let mut rate;
    let mut now;

    // index of the mutable triangle
    let mut index: usize = 0;

    // create a new image with white background
    init_image(&mut image);

    // create struct to memorize which point to be saved in each image
    let mut dummy_layer: Vec<Point> = vec![Point{x:0,y:0}; (width*height/2 + 1) as usize]; // the biggest triangle occupies the maximum of the size of the image. The +1 saves space to know how many pixel that triangles occupies e.g. Point{x:n_triangle, y:0}
    let mut written_pixels: Vec<Vec<Point>> = vec![dummy_layer.clone(); n_shapes]; // the first triangle is not needed to be counted, since it is chanced every time it is looked 

    let mut previous_image: Vec<RgbaImage> = vec![image.clone(); n_shapes];
    // draw(&mut image, &shapes, &new_shape, &mut previous_image, 0, n_shapes, &written_pixels, &mut dummy_layer, true); // sends the value 0 so it can pass through the all vector
    for i in 0..n_shapes {
        previous_image[i] = image.clone();
        draw_triangle(&shapes[i], &mut image, &mut written_pixels[i]);
    }

    // distance between the image and the target image
    let mut distance_matrix: Vec<u32> = vec![u32::MAX; (width*height) as usize];
    // init this matrix
    for x in 0..width {
        for y in 0..height {
            distance_matrix[(x*height+y) as usize] = color_distance(image.get_pixel(x, y), ref_image.get_pixel(x, y));   // get the initial distance between for each pixel
        }
    }

    // main loop, runs mutation, gets fitness (distance between 2 images), keeps or discards a mutation
    for i in 0..epochs{
        now = Instant::now();
        
        // mutate a shape and get a copy of the shapes vector
        mutate(&shapes, width, height, &mut index, &mut new_shape);
        
        // draw in the new image the vec of triangles with the mutated triangle
        draw(&mut image, &shapes, &new_shape, &mut previous_image, index, n_shapes, &written_pixels, &mut dummy_layer, false);
        
        // computes the distance between the new image and the reference image
        fitness(&image, &ref_image, &mut distance, &mut distance_matrix, &dummy_layer, &written_pixels[index], false);

        // if the new distance is better than the best distance, we accept the mutation
        if best_distance > distance {
            shapes[index] = new_shape.clone();
            fitness(&image, &ref_image, &mut distance, &mut distance_matrix, &dummy_layer, &written_pixels[index], true);   // important to be called before losing the previous shape information 
            written_pixels[index] = dummy_layer.clone();
            draw(&mut image, &shapes, &new_shape, &mut previous_image, index, n_shapes, &written_pixels, &mut dummy_layer, true);
            best_distance = distance;
        }

        rate = now.elapsed().as_millis(); // NEW
        if i%max(epochs/100,1) == 0 {println!("Mutation #{} - current distance: {:.3} - rate {:.3}", i, (best_distance as f32)/((width*height) as f32), 1000.0/rate as f32);}
        duration += rate;
    }

    println!("Computational time for {} epochs: {:.3} seconds with rate of {:.3} epoch/second", epochs, (duration as f32)/1000.0, (epochs as f32)/((duration as f32)/1000.0)); // NEW

    // draw(&mut image, &shapes, &new_shape, &mut previous_image, index, n_shapes, &written_pixels, &mut dummy_layer, false);
    for i in 0..n_shapes { draw_triangle(&shapes[i], &mut image, &mut written_pixels[i]); }
    _ = image.save(output_image_path);

    println!("Best fitness {}", (best_distance as f32)/((width*height) as f32));
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
fn random_triangle(w: u32, h: u32) -> Triangle {
    Triangle { 
        points: [
            random_point(w,h),
            random_point(w,h),
            random_point(w,h),
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
fn draw(image: &mut ImgRGBA, shapes: &Vec<Triangle>, new_shape: &Triangle, previous_image: &mut Vec<ImgRGBA>, index: usize, n_shapes: usize, written_pixels: &Vec<Vec<Point>>, dummy_layer: &mut Vec<Point>, save_best: bool) {
    *image = previous_image[index].clone();
    for i in index..n_shapes {
        if i!=index {overlapping_triangle(&shapes[i], image, &written_pixels[i]);}          // it sees which pixels it needs to update according for each triangle on top of the newest - 2nd and remaining iterations of the loop
        else {draw_triangle(&new_shape, image, dummy_layer)}         // it draws the new triangle on top of a the saved image before this layer - 1st iteration of the loop

        if save_best && (i != n_shapes-1) { previous_image[i+1] = image.clone();}
    }
}

// Fitness returns the average rgb color distance between 2 images
// it basically compares all pixels for 2 given images and returns
// a percentage that represents the similarities between the 2 images
// 0: the 2 images are the same
fn fitness(image: &ImgRGBA, ref_image: &ImgRGBA, fitness: &mut u32, distance_matrix: &mut Vec<u32>, dummy_layer: &Vec<Point>, old_layer: &Vec<Point>, save_matrix: bool){

    let w = image.width();
    let h = image.height();
    let mut distance_matrix_aux: Vec<u32> = (*distance_matrix).clone();
    *fitness = 0;
    let mut x;
    let mut y;

    // get the changes in the pixels modified by removing the current shape
    for pixel in 1..=old_layer[0].x as usize {
        x = old_layer[pixel].x;
        y = old_layer[pixel].y;
        distance_matrix_aux[(x * h + y) as usize] = color_distance(image.get_pixel(x, y), ref_image.get_pixel(x, y));
    }
    
    // get the changes in the pixels modified by removing the new hypothesis shape
    for pixel in 1..=dummy_layer[0].x as usize {
        x = dummy_layer[pixel].x;
        y = dummy_layer[pixel].y;
        distance_matrix_aux[(x * h + y) as usize] = color_distance(image.get_pixel(x, y), ref_image.get_pixel(x, y));
    }

    *fitness = 0;

    for x in 0..w{
        for y in 0..h{
            *fitness += distance_matrix_aux[(x * h + y) as usize];
        }
    }

    if save_matrix {*distance_matrix = distance_matrix_aux.clone()}
}

// color_distance returns the distance between 2 RGB colors
fn color_distance(color_1: &Rgba<u8>, color_2: &Rgba<u8>) -> u32{

    let r1 = color_1[0] as i32;
    let g1 = color_1[1] as i32;
    let b1 = color_1[2] as i32;

    let r2 = color_2[0] as i32;
    let g2 = color_2[1] as i32;
    let b2 = color_2[2] as i32;

    let result:f32 = ((r1 - r2)*(r1 - r2) + (g1 - g2)*(g1 - g2) + (b1 - b2)*(b1 - b2)) as f32;
    return ( result.sqrt() / 2.55 ) as u32;
}

// random_point creates and returns a random point
fn random_point(w: u32, h: u32) -> Point {
    Point{ 
        x: rand::thread_rng().gen_range(0..w),
        y: rand::thread_rng().gen_range(0..h) 
    }
}

// Mutate mutates a vertice coordinates or a color
fn mutate(shapes: & Vec<Triangle>, w: u32, h: u32, index: &mut usize, new_shape:&mut Triangle) {

    let point_mutation = rand::thread_rng().gen_range(0..2);
    *index = rand::thread_rng().gen_range(0..shapes.len());

    *new_shape = shapes[*index].clone();

    if point_mutation == 1 { // here we mutate a vertice

        let vertice_index = rand::thread_rng().gen_range(0..3);
        new_shape.points[vertice_index].x = rand::thread_rng().gen_range(0..w);
        new_shape.points[vertice_index].y = rand::thread_rng().gen_range(0..h);

    } else { // here we mutate a color
        let color_index = rand::thread_rng().gen_range(0..4);
        new_shape.color[color_index] = rand::thread_rng().gen_range(0..=255 as u8);
    }
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
fn draw_triangle(triangle: &Triangle, image: &mut ImgRGBA, dummy_layer: &mut Vec<Point>) {
    let x1 = triangle.points[0].x as i32;
    let y1 = triangle.points[0].y as i32;

    let x2 = triangle.points[1].x as i32;
    let y2 = triangle.points[1].y as i32;

    let x3 = triangle.points[2].x as i32;
    let y3 = triangle.points[2].y as i32;

    let xmin = min(x1, min(x2, x3));
    let xmax = max(x1, max(x2, x3));
    let ymin = min(y1, min(y2, y3));
    let ymax = max(y1, max(y2, y3));

    // pre compute all constant values
    let x21 = x2-x1;
    let y21 = y2-y1;
    let s21 = y21*x1-x21*y1;
    let x31 = x3-x1;
    let y31 = y3-y1;
    let s31 = y31*x1-x31*y1;
    let x32 = x3-x2;
    let y32 = y3-y2;
    let s32 = y32*x2-x32*y2;

    // save the information of the pixels to write
    let mut n_pixels = 0;

    for x in xmin .. xmax  {
        for y in ymin .. ymax {

            if (y*x21-x*y21 + s21 > 0) == (y*x31-x*y31 + s31 > 0) {continue;};
            if (y*x21-x*y21 + s21 > 0) != (y*x32-x*y32 + s32 > 0) {continue;};

            let current_pixel_color = image.get_pixel(x as u32, y as u32);
            let color = blend_color(current_pixel_color, &triangle.color);
            image.put_pixel(x as u32, y as u32, color);

            // save pixel coordinates
            n_pixels=n_pixels+1;
            dummy_layer[n_pixels].x = x as u32;
            dummy_layer[n_pixels].y = y as u32;
        }
    }

    dummy_layer[0].x = n_pixels as u32;
}

fn overlapping_triangle(triangle: &Triangle, image: &mut ImgRGBA, written_pixel: &Vec<Point>) {
    for pixel in 1..=written_pixel[0].x as usize {
        let current_pixel_color = image.get_pixel(written_pixel[pixel].x, written_pixel[pixel].y);
        let color = blend_color(current_pixel_color, &triangle.color);
        image.put_pixel(written_pixel[pixel].x, written_pixel[pixel].y, color);
    }
}