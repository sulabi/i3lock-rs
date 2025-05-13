use clap::Parser;
use fast_image_resize::{self as fir, IntoImageView};
use image::{DynamicImage, ImageBuffer, RgbImage};
use std::{ffi::c_ulong, io::Write, path::PathBuf, process::{Command, Stdio}};
use x11::xlib::{
    Display, Window, XDefaultRootWindow, XGetImage, XGetPixel, XGetWindowAttributes, XOpenDisplay,
    ZPixmap,
};

const ALL_PLANES: c_ulong = !0;

fn fast_resize(img: &DynamicImage, new_width: u32, new_height: u32) -> RgbImage {
    let mut dst_image = fir::images::Image::new(
        new_width,
        new_height,
        img.pixel_type().expect("Unsupported pixel type")
    );

    let mut resizer = fir::Resizer::new();

    resizer
        .resize(
            img,
            &mut dst_image,
            &Some(fir::ResizeOptions {
                algorithm: fast_image_resize::ResizeAlg::Convolution(
                    fast_image_resize::FilterType::Box
                ),
                ..Default::default()
            }),
        )
        .expect("Failed to resize image");

    ImageBuffer::from_raw(new_width, new_height, dst_image.buffer().to_vec()).unwrap()
}

fn box_blur(img: &RgbImage, radius: u32) -> RgbImage {
    image::imageops::blur(img, radius as f32)
}

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    image: Option<PathBuf>,

    #[arg(name = "i3lock_args", num_args = 0..)]
    i3lock_args: Vec<String>
}

fn main() {
    let args = Cli::parse();

    let display: *mut Display = unsafe { XOpenDisplay(std::ptr::null()) };
    if display.is_null() {
        eprintln!("Failed to open X display.");
        std::process::exit(1);
    }
    let root: Window = unsafe { XDefaultRootWindow(display) };

    let mut gwa = unsafe { std::mem::zeroed() };
    unsafe { XGetWindowAttributes(display, root, &mut gwa); }
    let screen_width = gwa.width as u32;
    let screen_height = gwa.height as u32;

    let img = {
        if let Some(img_path) = args.image {
            image::open(img_path).expect("Failed to load image")
        } else {
            let img = unsafe {
                XGetImage(
                    display,
                    root,
                    0,
                    0,
                    screen_width,
                    screen_height,
                    ALL_PLANES,
                    ZPixmap,
                )
            };
            let ximg = unsafe { &*img };
            let width = ximg.width as usize;
            let height = ximg.height as usize;

            if ximg.data.is_null() {
                eprintln!("Failed to load image data");
                std::process::exit(1);
            }

            let mut buffer = Vec::with_capacity(width * height * 3);

            for i in 0..height {
                for j in 0..width {
                    let pixel = unsafe { XGetPixel(img, j as i32, i as i32) };
                    let r = ((pixel & ximg.red_mask) >> ximg.red_mask.trailing_zeros()) as u8;
                    let g = ((pixel & ximg.green_mask) >> ximg.green_mask.trailing_zeros()) as u8;
                    let b = ((pixel & ximg.blue_mask) >> ximg.blue_mask.trailing_zeros()) as u8;

                    buffer.extend_from_slice(&[r, g, b]);
                }
            }
            DynamicImage::ImageRgb8(
                ImageBuffer::<image::Rgb<_>, _>::from_raw(width as u32, height as u32, buffer)
                    .unwrap()
            )
        }
    };

    let resized_img = {
        let small = fast_resize(&img, screen_width / 4, screen_height / 4);
        let blurred = box_blur(&small, 3);
        fast_resize(
            &DynamicImage::ImageRgb8(blurred),
            screen_width,
            screen_height
        )
    };

    let mut child = Command::new("i3lock")
        .arg("-i")
        .arg("/dev/stdin")
        .arg("--raw")
        .arg(format!("{}x{}:rgb", screen_width, screen_height))
        .args(args.i3lock_args)
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to start i3lock");

    let stdin = child.stdin.as_mut().expect("Failed to open stdin");

    let image_data = resized_img.as_raw();
    stdin
        .write_all(&image_data)
        .expect("Failed to write image data to i3lock");
}
