use std::{io, path::Path};

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use image::Rgb;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ColorType {
    AvgFgOnly,
    AvgBgOnly,
    FgTopBgDown,
    BgTopFgDown,
    #[default]
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputType {
    Term(ColorType),
    Html(ColorType),
    Svg(ColorType),
}

impl Default for OutputType {
    fn default() -> Self {
        Self::Term(ColorType::default())
    }
}

impl From<&Path> for OutputType {
    fn from(path: &Path) -> Self {
        let ext = path
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
    pub fn term() -> Self {
        Self::Term(ColorType::default())
    }
    pub fn html() -> Self {
        Self::Html(ColorType::default())
    }
    pub fn svg() -> Self {
        Self::Svg(ColorType::default())
    }
    pub fn set_color(&self, color: ColorType) -> Self {
        match self {
            Self::Term(_) => Self::Term(color),
            Self::Html(_) => Self::Html(color),
            Self::Svg(_) => Self::Svg(color),
        }
    }
    pub fn print_line<W: io::Write>(&self) -> fn(&mut W) -> io::Result<()> {
        match self {
            Self::Term(ColorType::None) => {
                fn line<W: io::Write>(stdout: &mut W) -> io::Result<()> {
                    stdout.write_all(b"\n")
                }
                line
            }
            Self::Term(_) => {
                fn line<W: io::Write>(stdout: &mut W) -> io::Result<()> {
                    execute!(stdout, ResetColor, Print("\n"))
                }
                line
            }
            Self::Html(ColorType::None) => {
                fn line<W: io::Write>(stdout: &mut W) -> io::Result<()> {
                    stdout.write_all(b"\n")
                }
                line
            }
            Self::Html(_) => {
                fn line<W: io::Write>(stdout: &mut W) -> io::Result<()> {
                    stdout.write_all(b"<br />\n")
                }
                line
            }
            Self::Svg(_) => todo!(),
        }
    }
    #[allow(clippy::type_complexity)]
    pub fn print_pixel<W: io::Write>(
        &self,
    ) -> fn(&mut W, (Rgb<u8>, Rgb<u8>), char) -> io::Result<()> {
        match self {
            Self::Term(ColorType::None) => {
                fn print<W: io::Write>(
                    stdout: &mut W,
                    _: (Rgb<u8>, Rgb<u8>),
                    ch: char,
                ) -> io::Result<()> {
                    execute!(stdout, Print(ch))
                }
                print
            }
            Self::Term(ColorType::AvgFgOnly) => {
                fn print<W: io::Write>(
                    stdout: &mut W,
                    (c1, c2): (Rgb<u8>, Rgb<u8>),
                    ch: char,
                ) -> io::Result<()> {
                    execute!(
                        stdout,
                        SetForegroundColor(rgb_to_true_color(avg_color(c1, c2))),
                        Print(ch)
                    )
                }
                print
            }
            Self::Term(ColorType::AvgBgOnly) => {
                fn print<W: io::Write>(
                    stdout: &mut W,
                    (c1, c2): (Rgb<u8>, Rgb<u8>),
                    ch: char,
                ) -> io::Result<()> {
                    execute!(
                        stdout,
                        SetBackgroundColor(rgb_to_true_color(avg_color(c1, c2))),
                        Print(ch)
                    )
                }
                print
            }
            Self::Term(ColorType::FgTopBgDown) => {
                fn print<W: io::Write>(
                    stdout: &mut W,
                    (c1, c2): (Rgb<u8>, Rgb<u8>),
                    ch: char,
                ) -> io::Result<()> {
                    execute!(
                        stdout,
                        SetBackgroundColor(rgb_to_true_color(c2)),
                        SetForegroundColor(rgb_to_true_color(c1)),
                        Print(ch)
                    )
                }
                print
            }
            Self::Term(ColorType::BgTopFgDown) => {
                fn print<W: io::Write>(
                    stdout: &mut W,
                    (c1, c2): (Rgb<u8>, Rgb<u8>),
                    ch: char,
                ) -> io::Result<()> {
                    execute!(
                        stdout,
                        SetBackgroundColor(rgb_to_true_color(c1)),
                        SetForegroundColor(rgb_to_true_color(c2)),
                        Print(ch)
                    )
                }
                print
            }
            Self::Html(ColorType::None) => {
                fn print<W: io::Write>(
                    stdout: &mut W,
                    _: (Rgb<u8>, Rgb<u8>),
                    ch: char,
                ) -> io::Result<()> {
                    write!(stdout, "{}", ch)
                }
                print
            }
            Self::Html(ColorType::AvgFgOnly) => {
                fn print<W: io::Write>(
                    stdout: &mut W,
                    (c1, c2): (Rgb<u8>, Rgb<u8>),
                    ch: char,
                ) -> io::Result<()> {
                    let color = avg_color(c1, c2);
                    write!(
                        stdout,
                        "<span style=\"color: {};\">{}</span>",
                        rgb_to_css_hex(color),
                        ch
                    )
                }
                print
            }
            Self::Html(ColorType::AvgBgOnly) => {
                fn print<W: io::Write>(
                    stdout: &mut W,
                    (c1, c2): (Rgb<u8>, Rgb<u8>),
                    ch: char,
                ) -> io::Result<()> {
                    let color = avg_color(c1, c2);
                    write!(
                        stdout,
                        "<span style=\"background-color: {};\">{}</span>",
                        rgb_to_css_hex(color),
                        ch
                    )
                }
                print
            }
            Self::Html(ColorType::FgTopBgDown) => {
                fn print<W: io::Write>(
                    stdout: &mut W,
                    (c1, c2): (Rgb<u8>, Rgb<u8>),
                    ch: char,
                ) -> io::Result<()> {
                    write!(
                        stdout,
                        "<span style=\"color: {};background-color: {};\">{}</span>",
                        rgb_to_css_hex(c1),
                        rgb_to_css_hex(c2),
                        ch
                    )
                }
                print
            }
            Self::Html(ColorType::BgTopFgDown) => {
                fn print<W: io::Write>(
                    stdout: &mut W,
                    (c1, c2): (Rgb<u8>, Rgb<u8>),
                    ch: char,
                ) -> io::Result<()> {
                    write!(
                        stdout,
                        "<span style=\"color: {};background-color: {};\">{}</span>",
                        rgb_to_css_hex(c2),
                        rgb_to_css_hex(c1),
                        ch
                    )
                }
                print
            }
            _ => todo!(),
        }
    }
}

impl OutputType {
    pub fn write_header<W: io::Write>(
        &self,
        file: &mut W,
        _width: u32,
        _height: u32,
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
                let data = format!(
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
                file.write_all(data.as_bytes())?;
            }
            Self::Svg(_) => todo!(),
            _ => {}
        }
        Ok(())
    }
    pub fn write_footer<W: io::Write>(&self, file: &mut W) -> io::Result<()> {
        match self {
            Self::Html(_) => file.write_all(
                br#"    </pre>
  </body>
</html>
"#,
            )?,
            Self::Svg(_) => todo!(),
            _ => {}
        }
        Ok(())
    }
}

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
}

fn rgb_to_css_hex(color: Rgb<u8>) -> String {
    format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2])
}
