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

    /// Style of Output Image
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
        let mut stdout = io::stdout().lock();
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
                    let style = Style::from([' ', '.', '-', '~', '+', '*', '%', '#', '@']);
                    style.print(
                        img,
                        &mut stdout,
                        |stdout| execute!(stdout, ResetColor, Print("\n")),
                        |stdout, (t, b), ch| {
                            execute!(
                                stdout,
                                SetForegroundColor(rgb_to_true_color(avg_color(t, b))),
                                Print(ch)
                            )
                        },
                    )?;
                }
                (StyleOps::Ascii, false) => {
                    let style = Style::from([' ', '.', '-', '~', '+', '*', '%', '#', '@']);
                    style.print(
                        img,
                        &mut stdout,
                        |stdout| execute!(stdout, Print("\n")),
                        |stdout, _, ch| execute!(stdout, Print(ch)),
                    )?;
                }
                (StyleOps::Block, true) => {
                    let style = Style::from([' ', '░', '▒', '▓']);
                    style.print(
                        img,
                        &mut stdout,
                        |stdout| execute!(stdout, Print("\n")),
                        |stdout, (c1, c2), ch| {
                            execute!(
                                stdout,
                                SetForegroundColor(rgb_to_true_color(avg_color(c1, c2))),
                                Print(ch)
                            )
                        },
                    )?;
                }
                (StyleOps::Block, false) => {
                    let style = Style::from([' ', '░', '▒', '▓']);
                    style.print(
                        img,
                        &mut stdout,
                        |stdout| execute!(stdout, Print("\n")),
                        |stdout, _, ch| execute!(stdout, Print(ch)),
                    )?;
                }
                (StyleOps::Pixel, true) => {
                    let style = Style::from(['▀']);
                    style.print(
                        img,
                        &mut stdout,
                        |stdout| execute!(stdout, ResetColor, Print("\n")),
                        |stdout, (top, button), ch| {
                            execute!(
                                stdout,
                                SetForegroundColor(rgb_to_true_color(top)),
                                SetBackgroundColor(rgb_to_true_color(button)),
                                Print(ch)
                            )
                        },
                    )?;
                }
                (StyleOps::Pixel, false) => {
                    print_img_not_colored(&mut stdout, img, vec![' ', '▀', '▞', '▟', '█'])?;
                }
                (StyleOps::Braills, true) => {
                    let style = Style::from([
                        [' ', '⠁', '⠉', '⠓', '⠛'],
                        ['⠄', '⠅', '⠩', '⠝', '⠟'],
                        ['⠤', '⠥', '⠭', '⠯', '⠽'],
                        ['⠴', '⠵', '⠽', '⠾', '⠿'],
                        ['⠶', '⠾', '⠾', '⠿', '⠿'],
                    ]);
                    style.print(
                        img,
                        &mut stdout,
                        |stdout| execute!(stdout, Print("\n")),
                        |stdout, (c1, c2), ch| {
                            execute!(
                                stdout,
                                SetForegroundColor(rgb_to_true_color(avg_color(c1, c2))),
                                Print(ch)
                            )
                        },
                    )?;
                }
                (StyleOps::Braills, false) => {
                    let style = Style::from([
                        [' ', '⠁', '⠉', '⠓', '⠛'],
                        ['⠄', '⠅', '⠩', '⠝', '⠟'],
                        ['⠤', '⠥', '⠭', '⠯', '⠽'],
                        ['⠴', '⠵', '⠽', '⠾', '⠿'],
                        ['⠶', '⠾', '⠾', '⠿', '⠿'],
                    ]);
                    style.print(
                        img,
                        &mut stdout,
                        |stdout| execute!(stdout, Print("\n")),
                        |stdout, _, ch| execute!(stdout, Print(ch)),
                    )?;
                }
                (StyleOps::Dots, true) => {
                    let style = Style::from([' ', '⠂', '⠒', '⠕', '⠞', '⠟', '⠿']);
                    style.print(
                        img,
                        &mut stdout,
                        |stdout| execute!(stdout, Print("\n")),
                        |stdout, (c1, c2), ch| {
                            execute!(
                                stdout,
                                SetForegroundColor(rgb_to_true_color(avg_color(c1, c2))),
                                Print(ch)
                            )
                        },
                    )?;
                }
                (StyleOps::Dots, false) => {
                    let style = Style::from([' ', '⠂', '⠒', '⠕', '⠞', '⠟', '⠿']);
                    style.print(
                        img,
                        &mut stdout,
                        |stdout| execute!(stdout, Print("\n")),
                        |stdout, _, ch| execute!(stdout, Print(ch)),
                    )?;
                }
                (StyleOps::Custom, false) => {
                    let input = self.args[0]
                        .clone()
                        .into_os_string()
                        .into_string()
                        .unwrap_or_else(|err| {
                            eprintln!("ERROR: envalid chars: '{:?}'", err);
                            std::process::exit(1);
                        });
                    let style = Style::from(input.chars().collect::<Vec<char>>());
                    style.print(
                        img,
                        &mut stdout,
                        |stdout| execute!(stdout, Print("\n")),
                        |stdout, _, ch| execute!(stdout, Print(ch)),
                    )?;
                }
                (StyleOps::Custom, true) => {
                    let input = self.args[0]
                        .clone()
                        .into_os_string()
                        .into_string()
                        .unwrap_or_else(|err| {
                            eprintln!("ERROR: envalid chars: '{:?}'", err);
                            std::process::exit(1);
                        });
                    let style = Style::from(input.chars().collect::<Vec<char>>());
                    style.print(
                        img,
                        &mut stdout,
                        |stdout| execute!(stdout, Print("\n")),
                        |stdout, (c1, c2), ch| {
                            execute!(
                                stdout,
                                SetForegroundColor(rgb_to_true_color(avg_color(c1, c2))),
                                Print(ch)
                            )
                        },
                    )?;
                }
            }
        }
        Ok(())
    }
}

#[inline(always)]
fn rgb_to_true_color(Rgb([r, g, b]): Rgb<u8>) -> Color {
    Color::Rgb { r, g, b }
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
            let index = std::cmp::min(((indent_t + indent_b) / 2) as usize, len - 1);
            let block = chars[index];
            execute!(stdout, Print(block))?;
        }
        execute!(stdout, Print("\n"))?;
    }
    Ok(())
}

struct Style(Vec<Vec<char>>);

impl<const R: usize, const C: usize> From<[[char; R]; C]> for Style {
    fn from(value: [[char; R]; C]) -> Self {
        Self(value.into_iter().map(|v| v.into_iter().collect()).collect())
    }
}

impl From<Vec<char>> for Style {
    fn from(value: Vec<char>) -> Self {
        Self(vec![value])
    }
}

impl<const R: usize> From<[char; R]> for Style {
    fn from(value: [char; R]) -> Self {
        Self(vec![value.into_iter().collect()])
    }
}

impl Style {
    fn get_char(&self, (Rgb([tr, tg, tb]), Rgb([br, bg, bb])): (Rgb<u8>, Rgb<u8>)) -> char {
        let r = self.0[0].len();
        let c = self.0.len();
        let ti = ((tr as u16 + tg as u16 + tb as u16) / 3) as u8;
        let bi = ((br as u16 + bg as u16 + bb as u16) / 3) as u8;
        let r_index = std::cmp::min((ti as usize * r) / u8::MAX as usize, r - 1);
        let c_index = std::cmp::min((bi as usize * c) / u8::MAX as usize, c - 1);
        self.0[c_index][r_index]
    }
    fn get_char_single_raw(&self, Rgb([tr, tg, tb]): Rgb<u8>, Rgb([br, bg, bb]): Rgb<u8>) -> char {
        let r = self.0[0].len();
        let ti = (tr as u16 + tg as u16 + tb as u16) / 3;
        let bi = (br as u16 + bg as u16 + bb as u16) / 3;
        let avg = (ti + bi) / 2;
        let r_index = std::cmp::min((avg as usize * r) / u8::MAX as usize, r - 1);
        self.0[0][r_index]
    }
    fn print<
        W: io::Write,
        G: Fn(&mut W) -> io::Result<()>,
        F: Fn(&mut W, (Rgb<u8>, Rgb<u8>), char) -> io::Result<()>,
    >(
        &self,
        img: DynamicImage,
        stdout: &mut W,
        line: G,
        f: F,
    ) -> io::Result<()> {
        let c = self.0.len();
        if c == 1 {
            for y in (0..img.height() - 1).step_by(2) {
                for x in 0..img.width() {
                    let t = img.get_pixel(x, y).to_rgb();
                    let b = img.get_pixel(x, y + 1).to_rgb();
                    let ch = self.get_char_single_raw(t, b);
                    f(stdout, (t, b), ch)?;
                }
                line(stdout)?;
            }
        } else {
            for y in (0..img.height() - 1).step_by(2) {
                for x in 0..img.width() {
                    let t = img.get_pixel(x, y).to_rgb();
                    let b = img.get_pixel(x, y + 1).to_rgb();
                    let ch = self.get_char((t, b));
                    f(stdout, (t, b), ch)?;
                }
                line(stdout)?;
            }
        }
        Ok(())
    }
}

#[inline(always)]
fn avg_color(c1: Rgb<u8>, c2: Rgb<u8>) -> Rgb<u8> {
    let r = (c1[0] as u16 + c2[0] as u16) / 2;
    let g = (c1[1] as u16 + c2[1] as u16) / 2;
    let b = (c1[2] as u16 + c2[2] as u16) / 2;
    Rgb([r as u8, g as u8, b as u8])
}
#[cfg(test)]
mod tests {
    use image::Rgb;

    use super::Style;

    #[test]
    fn test_style_get_char() {
        let st = Style::from([
            [' ', '⠁', '⠉', '⠓', '⠛'],
            ['⠄', '⠅', '⠩', '⠝', '⠟'],
            ['⠤', '⠥', '⠭', '⠯', '⠿'],
            ['⠴', '⠵', '⠽', '⠿', '⠿'],
            ['⠶', '⠾', '⠿', '⠿', '⠿'],
        ]);
        assert_eq!(st.get_char((Rgb([0, 0, 0]), Rgb([0, 0, 0]))), ' ');
        assert_eq!(st.get_char((Rgb([0xFF, 0xFF, 0xFF]), Rgb([0, 0, 0]))), '⠛');
        assert_eq!(st.get_char((Rgb([0xFE, 0xFE, 0xFE]), Rgb([0, 0, 0]))), '⠛');
        assert_eq!(
            st.get_char((Rgb([0xFF / 2, 0xFF / 2, 0xFF / 2]), Rgb([0, 0, 0]))),
            '⠉'
        );
        assert_eq!(st.get_char((Rgb([0, 0, 0]), Rgb([0xFF, 0xFF, 0xFF]))), '⠶');
        assert_eq!(st.get_char((Rgb([0, 0, 0]), Rgb([0xFE, 0xFE, 0xFE]))), '⠶');
        assert_eq!(
            st.get_char((Rgb([0, 0, 0]), Rgb([0xFF / 2, 0xFF / 2, 0xFF / 2]))),
            '⠤'
        );
        assert_eq!(
            st.get_char((Rgb([0xFF, 0xFF, 0xFF]), Rgb([0xFF, 0xFF, 0xFF]))),
            '⠿'
        );
        assert_eq!(
            st.get_char((Rgb([0xFE, 0xFE, 0xFE]), Rgb([0xFE, 0xFE, 0xFE]))),
            '⠿'
        );
        assert_eq!(
            st.get_char((
                Rgb([0xFF / 2, 0xFF / 2, 0xFF / 2]),
                Rgb([0xFF / 2, 0xFF / 2, 0xFF / 2])
            )),
            '⠭'
        );
    }
}
