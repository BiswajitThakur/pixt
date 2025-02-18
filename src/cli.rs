use clap::{Parser, ValueEnum};

use std::{io, path::PathBuf};

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageReader, Pixel, Rgb};

#[derive(Debug, Clone, Parser)]
pub struct Cli {
    /// Output width in terminal characters
    #[arg(short = 'w', long = "width")]
    width: Option<u32>,

    /// Output height in terminal characters
    #[arg(short = 'H', long = "height")]
    height: Option<u32>,

    /// Enable Color
    #[arg(short = 'c', long = "colored")]
    colored: bool,

    #[arg(
        short = 's',
        long = "style",
        value_enum,
        default_value_t = StyleOps::default(),
    )]
    style: StyleOps,

    #[arg(num_args = 1..)]
    args: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Default, ValueEnum)]
enum StyleOps {
    #[default]
    Pixel,
    Ascii,
    Block,
    Braills,
    Dots,
    Custom,
}

#[derive(Debug, Clone, Default, ValueEnum)]
enum ColorOps {
    None,
    #[default]
    TrueColor,
    Ansi256,
    Mono,
}

impl Cli {
    pub fn run(&self) -> io::Result<()> {
        let args = if self.style == StyleOps::Custom && self.args.len() < 2 {
            eprintln!("ERROR: Image Path Not Found");
            std::process::exit(1);
        } else if self.style == StyleOps::Custom {
            self.args.iter().skip(1).map(|v| v.clone()).collect()
        } else {
            self.args.clone()
        };
        let mut stdout = io::stdout();
        for path in args {
            let img = ImageReader::open(path)?.decode().unwrap_or_else(|err| {
                eprintln!("{}", err);
                std::process::exit(1);
            });
            let filter = FilterType::CatmullRom;
            let img = match (self.width, self.height) {
                (Some(width), Some(height)) => img.resize_exact(width, height, filter),
                (Some(width), None) => {
                    img.resize(width, (width * img.height()) / img.width(), filter)
                }
                (None, Some(height)) => img.resize(
                    std::cmp::min((height * img.width()) / img.height(), {
                        let (w, _) = crossterm::terminal::size()?;
                        w as u32
                    }),
                    height,
                    filter,
                ),
                (None, None) => {
                    let (w, _) = crossterm::terminal::size()?;
                    let h = (w as u32 * img.height()) / img.width();
                    img.resize(w as u32, h, filter)
                }
            };
            match (&self.style, &self.colored) {
                (StyleOps::Ascii, true) => {
                    print_img_colored(
                        &mut stdout,
                        img,
                        vec![' ', '.', '-', '~', '+', '*', '%', '#', '@'],
                    )?;
                }
                (StyleOps::Ascii, false) => {
                    print_img_not_colored(
                        &mut stdout,
                        img,
                        vec![' ', '.', '-', '~', '+', '*', '%', '#', '@'],
                    )?;
                }
                (StyleOps::Block, true) => {
                    print_img_colored(&mut stdout, img, vec![' ', '░', '▒', '▓'])?;
                }
                (StyleOps::Block, false) => {
                    print_img_not_colored(&mut stdout, img, vec![' ', '░', '▒', '▓'])?;
                }
                (StyleOps::Pixel, true) => {
                    print_img_pixel_colored(&mut stdout, img)?;
                }
                (StyleOps::Pixel, false) => {
                    print_img_not_colored(&mut stdout, img, vec![' ', '▀', '▞', '▟', '█'])?;
                }
                _ => {}
            }
        }
        Ok(())
    }
}
#[inline(always)]
fn rgb_to_true_color(Rgb([r, g, b]): Rgb<u8>) -> Color {
    Color::Rgb { r, g, b }
}

fn print_img_pixel_colored<W: io::Write>(stdout: &mut W, img: DynamicImage) -> io::Result<()> {
    for y in (0..img.height() - 1).step_by(2) {
        for x in 0..img.width() {
            let t = img.get_pixel(x, y).to_rgb();
            let b = img.get_pixel(x, y + 1).to_rgb();
            execute!(
                stdout,
                SetForegroundColor(rgb_to_true_color(t)),
                SetBackgroundColor(rgb_to_true_color(b)),
                Print("▀")
            )?;
        }
        execute!(stdout, ResetColor, Print("\n"))?;
    }
    Ok(())
}

fn print_img_not_colored<W: io::Write>(
    stdout: &mut W,
    img: DynamicImage,
    chars: Vec<char>,
) -> io::Result<()> {
    let len = chars.len();
    let s = u8::MAX / len as u8;
    for y in (0..img.height() - 1).step_by(2) {
        for x in 0..img.width() {
            let t = img.get_pixel(x, y).to_rgb();
            let b = img.get_pixel(x, y + 1).to_rgb();
            let indent_t = (t[0] / 3 + t[1] / 3 + t[2] / 3) / s;
            let indent_b = (b[0] / 3 + b[1] / 3 + b[2] / 3) / s;
            let block = chars[std::cmp::min(((indent_t + indent_b) / 2) as usize, len - 1)];
            execute!(stdout, Print(block))?;
        }
        execute!(stdout, ResetColor, Print("\n"))?;
    }
    Ok(())
}

fn print_img_colored<W: io::Write>(
    stdout: &mut W,
    img: DynamicImage,
    chars: Vec<char>,
) -> io::Result<()> {
    let len = chars.len();
    let s = u8::MAX / len as u8;
    for y in (0..img.height() - 1).step_by(2) {
        for x in 0..img.width() {
            let t = img.get_pixel(x, y).to_rgb();
            let b = img.get_pixel(x, y + 1).to_rgb();
            let avg = avg_color(t, b);
            let indent_t = (t[0] / 3 + t[1] / 3 + t[2] / 3) / s;
            let indent_b = (b[0] / 3 + b[1] / 3 + b[2] / 3) / s;
            let block = chars[std::cmp::min(((indent_t + indent_b) / 2) as usize, len - 1)];
            execute!(
                stdout,
                SetForegroundColor(rgb_to_true_color(avg)),
                Print(block)
            )?;
        }
        execute!(stdout, ResetColor, Print("\n"))?;
    }
    Ok(())
}

fn avg_color(c1: Rgb<u8>, c2: Rgb<u8>) -> Rgb<u8> {
    let r = (c1[0] as u16 + c2[0] as u16) / 2;
    let g = (c1[1] as u16 + c2[1] as u16) / 2;
    let b = (c1[2] as u16 + c2[2] as u16) / 2;
    Rgb([r as u8, g as u8, b as u8])
}
