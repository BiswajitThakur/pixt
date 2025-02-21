use clap::{Parser, ValueEnum};

use std::{
    fs,
    io::{self, BufWriter},
    path::PathBuf,
};

use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageReader, Pixel, Rgb};

use crate::output::{ColorType, OutputType};

#[derive(Debug, Clone, Parser)]
pub struct Cli {
    /// Output width in terminal characters
    #[arg(short = 'w', long = "width")]
    width: Option<u32>,

    /// Output height in terminal characters
    #[arg(short = 'H', long = "height")]
    height: Option<u32>,

    /// Enable colored output
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

    /// Optput
    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,

    /// Input file paths
    #[arg(num_args = 1..)]
    files: Vec<PathBuf>,
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
        if let Some(path) = &self.output {
            let file = fs::File::create_new(path).unwrap_or_else(|err| {
                eprintln!("{}", err);
                std::process::exit(1);
            });
            let stdout = BufWriter::new(file);
            render_app(stdout, self)?;
        } else {
            render_app(io::stdout(), self)?;
        }
        Ok(())
    }
}

fn render_image<W: io::Write>(
    mut style: Style<W>,
    out: &OutputType,
    img: DynamicImage,
) -> io::Result<()> {
    style.print_header(&img, |stdout, width, height| {
        out.write_header(stdout, width, height)
    })?;
    style.print(img, out.print_line(), out.print_pixel())?;
    style.print_footer(|stdout| out.write_footer(stdout))?;
    Ok(())
}

fn render_app<W: io::Write>(mut stdout: W, app: &Cli) -> io::Result<()> {
    // Extract image paths if the `--style | -s custom` option is provided in the CLI.
    // - If the `custom` style is selected but no image path is provided, print an error and exit.
    // - Otherwise, if `custom` is selected, skip the first argument (which may be the style
    //   option) and collect the rest as image paths.
    // - If a different style is selected, use all provided arguments as they are.
    let args = if app.style == StyleOps::Custom && app.files.len() < 2 {
        eprintln!("ERROR: Image Path Not Found");
        std::process::exit(1);
    } else if app.style == StyleOps::Custom {
        app.files.iter().skip(1).cloned().collect()
    } else {
        app.files.clone()
    };
    let mut out = OutputType::from(app.output.clone().unwrap_or_default().as_path());
    //let out = OutputType::Html;

    // Iterate over the provided image paths and process each image.
    // - Open the image file using `ImageReader`.
    // - Decode the image; if decoding fails, print the error and exit.
    for path in args {
        let img = ImageReader::open(path)?.decode().unwrap_or_else(|err| {
            eprintln!("{}", err);
            std::process::exit(1);
        });
        let filter = FilterType::CatmullRom;
        // Resize the image based on the provided width and height options:
        // - If both width and height are specified, resize the image exactly to those dimensions.
        // - If only width is specified, calculate the height to maintain the aspect ratio.
        // - If only height is specified, calculate the width to maintain the aspect ratio,
        //   ensuring it does not exceed the terminal width.
        // - If neither width nor height is specified, scale the image to fit within the terminal width
        //   while maintaining the aspect ratio.
        let img = match (app.width, app.height) {
            (Some(width), Some(height)) => img.resize_exact(width, height, filter),
            (Some(width), None) => img.resize(width, (width * img.height()) / img.width(), filter),
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
        // Handle image rendering based on style and color mode.
        // - `StyleOps` determines the image rendering style (e.g., ASCII, block, etc.).
        // - `bool` (`self.colored`) specifies whether to render in color or grayscale.
        match (&app.style, &app.colored) {
            (StyleOps::Ascii, true) => {
                out = out.set_color(ColorType::AvgFgOnly);
                let style =
                    Style::from((&mut stdout, [' ', '.', '-', '~', '+', '*', '%', '#', '@']));
                render_image(style, &out, img)?;
            }
            (StyleOps::Ascii, false) => {
                out = out.set_color(ColorType::None);
                let style =
                    Style::from((&mut stdout, [' ', '.', '-', '~', '+', '*', '%', '#', '@']));
                render_image(style, &out, img)?;
            }
            (StyleOps::Block, true) => {
                out = out.set_color(ColorType::AvgFgOnly);
                let style = Style::from((&mut stdout, [' ', '░', '▒', '▓']));
                render_image(style, &out, img)?;
            }
            (StyleOps::Block, false) => {
                out = out.set_color(ColorType::None);
                let style = Style::from((&mut stdout, [' ', '░', '▒', '▓']));
                render_image(style, &out, img)?;
            }
            (StyleOps::Pixel, true) => {
                out = out.set_color(ColorType::FgTopBgDown);
                let style = Style::from((&mut stdout, ['▀']));
                render_image(style, &out, img)?;
            }
            (StyleOps::Pixel, false) => {
                out = out.set_color(ColorType::None);
                let style = Style::from((&mut stdout, [' ', '▀', '▞', '▟', '█']));
                render_image(style, &out, img)?;
            }
            (StyleOps::Braills, true) => {
                out = out.set_color(ColorType::AvgFgOnly);
                let style = Style::from((
                    &mut stdout,
                    [
                        [' ', '⠁', '⠉', '⠓', '⠛'],
                        ['⠄', '⠅', '⠩', '⠝', '⠟'],
                        ['⠤', '⠥', '⠭', '⠯', '⠽'],
                        ['⠴', '⠵', '⠽', '⠾', '⠿'],
                        ['⠶', '⠾', '⠾', '⠿', '⠿'],
                    ],
                ));
                render_image(style, &out, img)?;
            }
            (StyleOps::Braills, false) => {
                out = out.set_color(ColorType::None);
                let style = Style::from((
                    &mut stdout,
                    [
                        [' ', '⠁', '⠉', '⠓', '⠛'],
                        ['⠄', '⠅', '⠩', '⠝', '⠟'],
                        ['⠤', '⠥', '⠭', '⠯', '⠽'],
                        ['⠴', '⠵', '⠽', '⠾', '⠿'],
                        ['⠶', '⠾', '⠾', '⠿', '⠿'],
                    ],
                ));
                render_image(style, &out, img)?;
            }
            (StyleOps::Dots, true) => {
                out = out.set_color(ColorType::AvgFgOnly);
                let style = Style::from((&mut stdout, [' ', '⠂', '⠒', '⠕', '⠞', '⠟', '⠿']));
                render_image(style, &out, img)?;
            }
            (StyleOps::Dots, false) => {
                out = out.set_color(ColorType::None);
                let style = Style::from((&mut stdout, [' ', '⠂', '⠒', '⠕', '⠞', '⠟', '⠿']));
                render_image(style, &out, img)?;
            }
            (StyleOps::Custom, false) => {
                let input = app.files[0]
                    .clone()
                    .into_os_string()
                    .into_string()
                    .unwrap_or_else(|err| {
                        eprintln!("ERROR: envalid chars: '{:?}'", err);
                        std::process::exit(1);
                    });
                out = out.set_color(ColorType::None);
                let style = Style::from((&mut stdout, input.chars().collect::<Vec<char>>()));
                render_image(style, &out, img)?;
            }
            (StyleOps::Custom, true) => {
                let input = app.files[0]
                    .clone()
                    .into_os_string()
                    .into_string()
                    .unwrap_or_else(|err| {
                        eprintln!("ERROR: envalid chars: '{:?}'", err);
                        std::process::exit(1);
                    });
                out = out.set_color(ColorType::AvgFgOnly);
                let style = Style::from((&mut stdout, input.chars().collect::<Vec<char>>()));
                render_image(style, &out, img)?;
            }
        }
    }
    Ok(())
}

/// A struct representing a 2D character-based style for rendering images.
///
/// - Useful for ASCII art generation, where two pixels (upper and bottom)  
///   are combined into a single character (since each character is not a square).
struct Style<'a, O: io::Write>(Vec<Vec<char>>, &'a mut O);

impl<'a, const R: usize, const C: usize, O: io::Write> From<(&'a mut O, [[char; R]; C])>
    for Style<'a, O>
{
    fn from((stdout, value): (&'a mut O, [[char; R]; C])) -> Self {
        Self(
            value.into_iter().map(|v| v.into_iter().collect()).collect(),
            stdout,
        )
    }
}

impl<'a, O: io::Write> From<(&'a mut O, Vec<char>)> for Style<'a, O> {
    fn from((stdout, value): (&'a mut O, Vec<char>)) -> Self {
        Self(vec![value], stdout)
    }
}

impl<'a, const R: usize, O: io::Write> From<(&'a mut O, [char; R])> for Style<'a, O> {
    fn from((stdout, value): (&'a mut O, [char; R])) -> Self {
        Self(vec![value.into_iter().collect()], stdout)
    }
}

impl<O: io::Write> Style<'_, O> {
    /// Returns a character representing the brightness levels of two pixels.
    fn get_char(&self, (Rgb([tr, tg, tb]), Rgb([br, bg, bb])): (Rgb<u8>, Rgb<u8>)) -> char {
        let rows = self.0.len(); // Number of character rows
        let cols = self.0[0].len(); // Number of character columns

        // Compute grayscale intensity for both pixels using an average of RGB values
        let top_intensity = ((tr as u16 + tg as u16 + tb as u16) / 3) as u8;
        let bottom_intensity = ((br as u16 + bg as u16 + bb as u16) / 3) as u8;

        // Map intensity to row and column indices, ensuring they stay within bounds
        let row_index = std::cmp::min((top_intensity as usize * cols) / u8::MAX as usize, cols - 1);
        let col_index = std::cmp::min(
            (bottom_intensity as usize * rows) / u8::MAX as usize,
            rows - 1,
        );

        self.0[col_index][row_index]
    }
    /// Returns a character representing the average brightness of two pixels.
    fn get_char_single_raw(&self, Rgb([tr, tg, tb]): Rgb<u8>, Rgb([br, bg, bb]): Rgb<u8>) -> char {
        let cols = self.0[0].len();

        let top_intensity = (tr as u16 + tg as u16 + tb as u16) / 3;
        let bottom_intensity = (br as u16 + bg as u16 + bb as u16) / 3;

        let avg_intensity = (top_intensity + bottom_intensity) / 2;

        let col_index = std::cmp::min((avg_intensity as usize * cols) / u8::MAX as usize, cols - 1);

        self.0[0][col_index]
    }
    /// Renders an image as ASCII-style characters and prints it to the given output.
    ///
    /// # Parameters:
    /// - `stdout`: The output stream to print the characters.
    /// - `img`: The input image to render.
    /// - `line`: A function that writes a new line after each row.
    /// - `print_pixel`: A function that processes and prints each character.
    fn print<
        G: Fn(&mut O) -> io::Result<()>,
        F: Fn(&mut O, (Rgb<u8>, Rgb<u8>), char) -> io::Result<()>,
    >(
        &mut self,
        img: DynamicImage,
        line: G,
        print_pixel: F,
    ) -> io::Result<()> {
        let c = self.0.len();
        if c == 1 {
            for y in (0..img.height() - 1).step_by(2) {
                for x in 0..img.width() {
                    let t = img.get_pixel(x, y).to_rgb();
                    let b = img.get_pixel(x, y + 1).to_rgb();
                    let ch = self.get_char_single_raw(t, b);
                    print_pixel(self.1, (t, b), ch)?;
                }
                line(self.1)?;
            }
        } else {
            for y in (0..img.height() - 1).step_by(2) {
                for x in 0..img.width() {
                    let t = img.get_pixel(x, y).to_rgb();
                    let b = img.get_pixel(x, y + 1).to_rgb();
                    let ch = self.get_char((t, b));
                    print_pixel(self.1, (t, b), ch)?;
                }
                line(self.1)?;
            }
        }
        Ok(())
    }
    fn print_header<F: Fn(&mut O, u32, u32) -> io::Result<()>>(
        &mut self,
        img: &DynamicImage,
        f: F,
    ) -> io::Result<()> {
        f(self.1, img.width(), img.height())
    }
    fn print_footer<F: Fn(&mut O) -> io::Result<()>>(&mut self, f: F) -> io::Result<()> {
        f(self.1)
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use image::Rgb;

    use super::Style;

    #[test]
    fn test_style_get_char() {
        let mut stdout = io::sink();
        let st = Style::from((
            &mut stdout,
            [
                [' ', '⠁', '⠉', '⠓', '⠛'],
                ['⠄', '⠅', '⠩', '⠝', '⠟'],
                ['⠤', '⠥', '⠭', '⠯', '⠿'],
                ['⠴', '⠵', '⠽', '⠿', '⠿'],
                ['⠶', '⠾', '⠿', '⠿', '⠿'],
            ],
        ));
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
