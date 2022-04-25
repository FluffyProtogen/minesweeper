use std::{fs::File, io::Read};

use image::{codecs::png::PngEncoder, DynamicImage, GenericImage, ImageEncoder, Rgba};
use rusttype::{point, Font, Scale};
use tiny_skia::Pixmap;

lazy_static! {
    pub static ref ROBOTO: Font<'static> = Font::try_from_vec({
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path.push("assets");
        path.push("fonts");
        path.push("Roboto-Bold.ttf");

        let file = File::open(path).unwrap();
        file.bytes().flatten().collect::<Vec<_>>()
    })
    .unwrap();
    pub static ref EB_GARAMOND: Font<'static> = Font::try_from_vec({
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path.push("assets");
        path.push("fonts");
        path.push("EBGaramond-SemiBold.ttf");

        let file = File::open(path).unwrap();
        file.bytes().flatten().collect::<Vec<_>>()
    })
    .unwrap();
}

pub fn text_to_pixmap(text: &str, font: &Font, size: f32, color: (u8, u8, u8)) -> Pixmap {
    //from https://github.com/redox-os/rusttype/blob/master/dev/examples/image.rs

    let scale = Scale::uniform(size);

    let metrics = font.v_metrics(scale);

    let glyphs: Vec<_> = font
        .layout(text, scale, point(10.0, metrics.ascent))
        .collect();

    let glyphs_height = (metrics.ascent - metrics.descent).ceil() as u32;
    let glyphs_width = {
        let min_x = glyphs
            .first()
            .map(|g| g.pixel_bounding_box().unwrap().min.x)
            .unwrap();
        let max_x = glyphs
            .last()
            .map(|g| g.pixel_bounding_box().unwrap().max.x)
            .unwrap();
        (max_x - min_x) as u32
    };

    let mut image = DynamicImage::new_rgba8(glyphs_width + 40, glyphs_height);

    for glyph in glyphs {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                image.put_pixel(
                    x + bounding_box.min.x as u32,
                    y + bounding_box.min.y as u32,
                    Rgba([color.0, color.1, color.2, (v * 255.0) as u8]),
                )
            });
        }
    }
    //
    let mut bytes = Vec::new();

    let encoder = PngEncoder::new(&mut bytes);

    encoder
        .write_image(
            image.as_rgba8().unwrap(),
            image.width(),
            image.height(),
            image::ColorType::Rgba8,
        )
        .unwrap();

    Pixmap::decode_png(&bytes).unwrap()
}
