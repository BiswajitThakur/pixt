use std::{io, path::Path};

#[cfg(not(target_arch = "wasm32"))]
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use image::{DynamicImage, GenericImageView, Pixel as _, Rgb};
use wasm_bindgen::JsValue;

pub struct PixtImg {
    data: PixtData,
    out_type: OutputType,
}

impl PixtImg {
    pub fn new<T: IntoPixtData>(data: T, out_type: OutputType) -> Self {
        Self {
            data: data.into(),
            out_type,
        }
    }
    pub fn print(&self, img: &DynamicImage, mut out: impl io::Write) -> io::Result<()> {
        self.out_type
            .write_header(img.width(), img.height(), &mut out)?;
        for line in self.data.chars(img) {
            for p in line {
                let print = self.out_type.print_pixel();
                print(&mut out, p)?;
            }
            let println = self.out_type.print_line();
            println(&mut out)?;
        }
        Ok(())
    }
}

pub struct PixtData {
    data: Vec<Vec<char>>,
}

impl PixtData {
    pub fn new<T: IntoPixtData>(data: T) -> Self {
        Self {
            data: data.into_pixt_data(),
        }
    }
    pub fn set_pixel_data<T: IntoPixtData>(&mut self, data: T) -> &mut Self {
        self.data = data.into_pixt_data();
        self
    }
    /// Returns a character representing the brightness levels of two pixels.
    fn get_char(&self, Rgb([tr, tg, tb]): Rgb<u8>, Rgb([br, bg, bb]): Rgb<u8>) -> char {
        let rows = self.data.len(); // Number of character rows
        let cols = self.data[0].len(); // Number of character columns

        // Compute grayscale intensity for both pixels using an average of RGB values
        let top_intensity = ((tr as u16 + tg as u16 + tb as u16) / 3) as u8;
        let bottom_intensity = ((br as u16 + bg as u16 + bb as u16) / 3) as u8;

        // Map intensity to row and column indices, ensuring they stay within bounds
        let row_index = std::cmp::min((top_intensity as usize * cols) / u8::MAX as usize, cols - 1);
        let col_index = std::cmp::min(
            (bottom_intensity as usize * rows) / u8::MAX as usize,
            rows - 1,
        );

        self.data[col_index][row_index]
    }
    /// Returns a character representing the average brightness of two pixels.
    fn get_char_single_raw(&self, Rgb([tr, tg, tb]): Rgb<u8>, Rgb([br, bg, bb]): Rgb<u8>) -> char {
        let cols = self.data[0].len();

        let top_intensity = (tr as u16 + tg as u16 + tb as u16) / 3;
        let bottom_intensity = (br as u16 + bg as u16 + bb as u16) / 3;

        let avg_intensity = (top_intensity + bottom_intensity) / 2;

        let col_index = std::cmp::min((avg_intensity as usize * cols) / u8::MAX as usize, cols - 1);

        self.data[0][col_index]
    }
}

pub struct Pixel {
    pub x: u32,
    pub y: u32,
    pub color: (u8, u8, u8),
}

impl From<Pixel> for [u8; 3] {
    fn from(value: Pixel) -> Self {
        let Pixel {
            x: _,
            y: _,
            color: (r, g, b),
        } = value;
        [r, g, b]
    }
}
impl From<Pixel> for (u8, u8, u8) {
    fn from(value: Pixel) -> Self {
        let Pixel {
            x: _,
            y: _,
            color: v,
        } = value;
        v
    }
}
impl PixtData {
    pub fn chars(
        &self,
        img: &DynamicImage,
    ) -> impl Iterator<Item = impl Iterator<Item = (char, Pixel, Pixel)>> {
        struct ItrImgOuter<'a, 'b> {
            y: u32,
            img: &'a DynamicImage,
            pixt_img: &'b PixtData,
        }
        impl<'a, 'b> ItrImgOuter<'a, 'b> {
            fn new(img: &'a DynamicImage, pixt_img: &'b PixtData) -> Self {
                Self {
                    y: 0,
                    img,
                    pixt_img,
                }
            }
        }

        struct ItrImgInner<'a, 'b> {
            x: u32,
            y: u32,
            img: &'a DynamicImage,
            pixt_img: &'b PixtData,
        }
        impl Iterator for ItrImgInner<'_, '_> {
            type Item = (char, Pixel, Pixel);
            fn next(&mut self) -> Option<Self::Item> {
                if self.x >= self.img.width() || self.y >= self.img.height() {
                    return None;
                }
                let t = self.img.get_pixel(self.x, self.y).to_rgb();
                let b = self.img.get_pixel(self.x, self.y + 1).to_rgb();
                let p1 = Pixel {
                    x: self.x,
                    y: self.y,
                    color: unwrap_rgb(t),
                };
                let p2 = Pixel {
                    x: self.x,
                    y: self.y + 1,
                    color: unwrap_rgb(b),
                };
                self.x += 1;
                if self.pixt_img.data.len() == 1 {
                    Some((self.pixt_img.get_char_single_raw(t, b), p1, p2))
                } else {
                    Some((self.pixt_img.get_char(t, b), p1, p2))
                }
            }
        }
        impl<'a, 'b> Iterator for ItrImgOuter<'a, 'b> {
            type Item = ItrImgInner<'a, 'b>;
            fn next(&mut self) -> Option<Self::Item> {
                let y = self.y;
                if y + 1 >= self.img.height() {
                    return None;
                }
                self.y += 1;
                Some(ItrImgInner {
                    x: 0,
                    y,
                    img: self.img,
                    pixt_img: self.pixt_img,
                })
            }
        }
        ItrImgOuter::new(img, self)
    }
}

impl<T: IntoPixtData> From<T> for PixtData {
    fn from(value: T) -> Self {
        Self {
            data: value.into_pixt_data(),
        }
    }
}

pub trait IntoPixtData {
    fn into_pixt_data(self) -> Vec<Vec<char>>;
}

impl IntoPixtData for Vec<Vec<char>> {
    fn into_pixt_data(self) -> Vec<Vec<char>> {
        self
    }
}

impl<const N: usize> IntoPixtData for [Vec<char>; N] {
    fn into_pixt_data(self) -> Vec<Vec<char>> {
        self.into()
    }
}

impl<const N: usize, const M: usize> IntoPixtData for [[char; M]; N] {
    fn into_pixt_data(self) -> Vec<Vec<char>> {
        self.into_iter().map(|v| v.into()).collect()
    }
}

impl<const N: usize> IntoPixtData for [char; N] {
    fn into_pixt_data(self) -> Vec<Vec<char>> {
        vec![self.into()]
    }
}

impl IntoPixtData for Vec<char> {
    fn into_pixt_data(self) -> Vec<Vec<char>> {
        vec![self]
    }
}

fn unwrap_rgb(c: Rgb<u8>) -> (u8, u8, u8) {
    let Rgb([a, b, c]) = c;
    (a, b, c)
}

/// Output color type
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ColorType {
    /// Avg of upper and lower applied as forground (font) color
    AvgFgOnly,
    /// Avg of upper and lower applied as background color
    AvgBgOnly,
    /// upper pixel color as forground, lower pixel color as background
    FgTopBgDown,
    /// upper pixel color as background, lower pixel color as forground
    BgTopFgDown,
    /// default color
    #[default]
    None,
}

/// Output type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputType {
    Text(ColorType),
    Term(ColorType),
    Html(ColorType),
    Svg(ColorType), // TODO: Implement proper SVG rendering
}

impl Default for OutputType {
    fn default() -> Self {
        Self::Term(ColorType::default())
    }
}

impl<T: AsRef<Path>> From<T> for OutputType {
    fn from(path: T) -> Self {
        let ext = path
            .as_ref()
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        let color = ColorType::default();
        match ext.to_lowercase().as_str() {
            "html" | "htm" => Self::Html(color),
            "svg" => Self::Svg(color),
            _ => Self::Term(color),
        }
    }
}

impl OutputType {
    pub const fn text() -> Self {
        Self::Text(ColorType::None)
    }
    pub fn term() -> Self {
        Self::Term(ColorType::default())
    }
    pub fn html() -> Self {
        Self::Html(ColorType::default())
    }
    pub fn svg() -> Self {
        Self::Svg(ColorType::default())
    }
    pub fn color(mut self, color: ColorType) -> Self {
        self = match self {
            Self::Text(_) => Self::Text(color),
            Self::Term(_) => Self::Term(color),
            Self::Html(_) => Self::Html(color),
            Self::Svg(_) => Self::Svg(color),
        };
        self
    }

    pub fn print_line<W>(&self) -> impl Fn(W) -> io::Result<()>
    where
        W: io::Write,
    {
        match self {
            Self::Text(_) | Self::Term(ColorType::None) => |mut w: W| w.write_all(b"\n"),
            Self::Term(_) => |mut stdout: W| {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    execute!(stdout, ResetColor, Print("\n"))
                }
                #[cfg(target_arch = "wasm32")]
                {
                    Err(io::Error::other("This features is not available for web"))
                }
            },
            Self::Html(ColorType::None) => |mut w: W| w.write_all(b"\n"),
            Self::Html(_) => |mut stdout: W| stdout.write_all(b"<br />\n"),
            Self::Svg(ColorType::None) => |_| todo!(),
            Self::Svg(_) => |_| todo!(),
        }
    }
    #[allow(clippy::type_complexity)]
    pub fn print_pixel<W: io::Write>(&self) -> impl Fn(W, (char, Pixel, Pixel)) -> io::Result<()> {
        match self {
            Self::Text(_) | Self::Term(ColorType::None) => |mut out: W, (v, _, _)| {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    execute!(out, Print(v))
                }
                #[cfg(target_arch = "wasm32")]
                {
                    write!(out, "{}", v)
                }
            },
            Self::Term(ColorType::AvgFgOnly) => |mut out: W, (ch, c1, c2): (char, Pixel, Pixel)| {
                let c1: [u8; 3] = c1.into();
                let c2: [u8; 3] = c2.into();
                #[cfg(not(target_arch = "wasm32"))]
                {
                    execute!(
                        out,
                        SetForegroundColor(rgb_to_true_color(avg_color(c1, c2))),
                        Print(ch)
                    )
                }
                #[cfg(target_arch = "wasm32")]
                {
                    Err(io::Error::other("This features is not available for web"))
                }
            },
            Self::Term(ColorType::AvgBgOnly) => |mut out: W, (ch, c1, c2): (char, Pixel, Pixel)| {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    execute!(
                        out,
                        SetBackgroundColor(rgb_to_true_color(avg_color(c1.into(), c2.into()))),
                        Print(ch)
                    )
                }
                #[cfg(target_arch = "wasm32")]
                {
                    Err(io::Error::other("This features is not available for web"))
                }
            },
            Self::Term(ColorType::FgTopBgDown) => {
                |mut out: W, (ch, c1, c2): (char, Pixel, Pixel)| {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        execute!(
                            out,
                            SetBackgroundColor(rgb_to_true_color(c2)),
                            SetForegroundColor(rgb_to_true_color(c1)),
                            Print(ch)
                        )
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        Err(io::Error::other("This features is not available for web"))
                    }
                }
            }
            Self::Term(ColorType::BgTopFgDown) => {
                |mut out: W, (ch, c1, c2): (char, Pixel, Pixel)| {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        execute!(
                            out,
                            SetBackgroundColor(rgb_to_true_color(c1)),
                            SetForegroundColor(rgb_to_true_color(c2)),
                            Print(ch)
                        )
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        Err(io::Error::other("This features is not available for web"))
                    }
                }
            }
            Self::Html(ColorType::None) => {
                |mut out: W, (ch, _, _): (char, Pixel, Pixel)| write!(out, "{}", ch)
            }
            Self::Html(ColorType::AvgFgOnly) => |mut out: W, (ch, c1, c2): (char, Pixel, Pixel)| {
                let color = avg_color(c1.into(), c2.into());
                write!(
                    out,
                    "<span style=\"color: {};\">{}</span>",
                    rgb_to_css_hex(color),
                    ch
                )
            },
            Self::Html(ColorType::AvgBgOnly) => |mut out: W, (ch, c1, c2): (char, Pixel, Pixel)| {
                let color = avg_color(c1.into(), c2.into());
                write!(
                    out,
                    "<span style=\"background-color:{};\">{}</span>",
                    rgb_to_css_hex(color),
                    ch
                )
            },
            Self::Html(ColorType::FgTopBgDown) => {
                |mut out: W, (ch, c1, c2): (char, Pixel, Pixel)| {
                    write!(
                        out,
                        "<span style=\"color:{};background-color:{};\">{}</span>",
                        rgb_to_css_hex(c1),
                        rgb_to_css_hex(c2),
                        ch
                    )
                }
            }
            Self::Html(ColorType::BgTopFgDown) => {
                |mut out: W, (ch, c1, c2): (char, Pixel, Pixel)| {
                    write!(
                        out,
                        "<span style=\"color:{};background-color:{};\">{}</span>",
                        rgb_to_css_hex(c2),
                        rgb_to_css_hex(c1),
                        ch
                    )
                }
            }
            Self::Svg(ColorType::None) => {
                todo!()
            }
            Self::Svg(ColorType::AvgFgOnly) => {
                todo!()
            }
            _ => todo!(),
        }
    }
    pub fn write_header<W: io::Write>(
        &self,
        _width: u32,
        _height: u32,
        mut out: W,
    ) -> io::Result<()> {
        match self {
            Self::Html(color) => {
                let margin = 0;
                let padding = 0;
                let font_size = 10; // px
                let line_height = match color {
                    ColorType::None => 1.2,
                    _ => 0.6,
                };
                let buf = format!(
                    "<!DOCTYPE html>
<html lang=\"en\">
  <head>
    <meta charset=\"UTF-8\">
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
    <style>
    * {{
        color: #fff;
        background-color: #191919;
        font-family: monospace;
    }}
    pre {{
        line-height: {line_height};
        margin: {margin};
        padding: {padding};
        font-size: {font_size}px;
    }}
    </style>
  </head>
  <body>
    <pre>"
                );
                out.write_all(buf.as_bytes())
            }
            Self::Svg(_) => {
                todo!()
            }
            _ => Ok(()),
        }
    }

    pub fn write_footer<W: io::Write>(&self, file: &mut W) -> io::Result<()> {
        match self {
            Self::Html(_) => file.write_all(b"    </pre>\n  </body>\n</html>\n")?,
            Self::Svg(_) => {
                todo!()
                // file.write_all(b"    </text>\n</svg>")?;
            }
            _ => {}
        }
        Ok(())
    }
}

/// Convert an `Rgb<u8>` value to a `Color::Rgb` type for terminal rendering.
#[inline(always)]
#[cfg(not(target_arch = "wasm32"))]
fn rgb_to_true_color<T: Into<[u8; 3]>>(color: T) -> Color {
    let [r, g, b] = color.into();
    Color::Rgb { r, g, b }
}
/*
/// Convert an `Rgb<u8>` value to a `Color::Rgb` type for terminal rendering.
#[inline(always)]
fn rgb_to_true_color(Rgb([r, g, b]): Rgb<u8>) -> Color {
    Color::Rgb { r, g, b }
}
#[inline(always)]
fn avg_color(c1: Rgb<u8>, c2: Rgb<u8>) -> Rgb<u8> {
    let r = (c1[0] as u16 + c2[0] as u16) / 2;
    let g = (c1[1] as u16 + c2[1] as u16) / 2;
    let b = (c1[2] as u16 + c2[2] as u16) / 2;
    Rgb([r as u8, g as u8, b as u8])
}*/
#[inline(always)]
fn avg_color([r1, g1, b1]: [u8; 3], [r2, g2, b2]: [u8; 3]) -> [u8; 3] {
    let r = (r1 as u16 + r2 as u16) / 2;
    let g = (g1 as u16 + g2 as u16) / 2;
    let b = (b1 as u16 + b2 as u16) / 2;
    [r as u8, g as u8, b as u8]
}
fn rgb_to_css_hex<T: Into<[u8; 3]>>(color: T) -> String {
    let color: [u8; 3] = color.into();
    unsafe {
        format!(
            "#{:02X}{:02X}{:02X}",
            color.get_unchecked(0),
            color.get_unchecked(1),
            color.get_unchecked(2)
        )
    }
}
