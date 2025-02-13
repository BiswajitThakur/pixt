use std::io;

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

use image::{imageops::FilterType, GenericImageView, ImageReader, Pixel};

fn main() -> io::Result<()> {
    let file_name: String = std::env::args().nth(1).unwrap();
    let (term_x, _) = crossterm::terminal::size().unwrap();
    let img = ImageReader::open(&file_name)?.decode().unwrap();
    let (img_x, img_y) = img.dimensions();
    let term_y = (term_x as u32 * img_y) / img_x;
    let img = img.resize(term_x as u32, term_y, FilterType::CatmullRom);
    let mut stdout = std::io::stdout();
    let (img_x, img_y) = img.dimensions();
    for y in (0..img_y).step_by(2) {
        for x in 0..img_x {
            let t = img.get_pixel(x, y).to_rgb();
            let b = img.get_pixel(x, std::cmp::min(y + 1, img_y - 1)).to_rgb();
            execute!(
                stdout,
                SetForegroundColor(Color::Rgb {
                    r: t[0],
                    g: t[1],
                    b: t[2]
                }),
                SetBackgroundColor(Color::Rgb {
                    r: b[0],
                    g: b[1],
                    b: b[2]
                }),
                Print("▀")
            )
            .unwrap();
        }
        execute!(stdout, ResetColor, Print("\n")).unwrap();
    }
    Ok(())
}
