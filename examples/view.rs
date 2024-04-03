use sdl2::pixels::{self, PixelFormatEnum};
use std::{env, fs, time::Duration};

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Use {} file.avi", args[0]);
        return;
    }

    let data = fs::read(&args[1]).unwrap();
    let (avi, frames) = avi_rs::read_avi(&data).unwrap();
    println!("AVI: {:#?}", avi);
    println!("frames: {}", frames.len());

    let time_between_frames = avi.avi_header().unwrap().time_between_frames;
    println!("fps: {}", time_between_frames);

    let stream_format = avi.stream_format_vid().unwrap();

    let width = stream_format.image_width as u32;
    let height = stream_format.image_height as u32;

    println!("compression_type: {}", stream_format.compression_type);

    // if stream_format.compression_type != "MPNG" {
    //     println!("Only MPNG supported");
    //     return;
    // }

    let window = video_subsystem
        .window(&args[1], width, height)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, width, height)
        .unwrap();
    canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));

    for (_i, frame) in frames.iter().enumerate() {
        // let mut file = File::create(format!("out/frame{}", i)).unwrap();
        // file.write_all(&frame.0).unwrap();
        // let image = image::load_from_memory(&frame.0).unwrap();
        // let bytes = image.as_bytes();
        let bytes = &frame.0;

        texture.update(None, &bytes, width as usize * 3).unwrap();
        canvas.clear();

        let rect = sdl2::rect::Rect::new(0, 0, width, height);
        canvas.copy(&texture, None, Some(rect)).unwrap();
        canvas.present();

        std::thread::sleep(Duration::new(0, time_between_frames as u32));
    }
}
