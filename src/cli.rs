use clap::{Parser, ValueEnum};
use pixt::{
    img::{ColorType, OutputType, PixtImg},
    style::ImgStyle,
};

use std::{
    fs,
    io::{self, BufReader, BufWriter, Read},
    path::PathBuf,
};

use image::{ImageReader, imageops::FilterType};

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

    /// Optput path.<txt|html|svg>
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
    FromFile,
}

impl Cli {
    pub fn run(&self) -> io::Result<()> {
        if let Some(path) = &self.output {
            let file = fs::File::create(path).unwrap_or_else(|err| {
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

fn render_app<W: io::Write>(mut stdout: W, app: &Cli) -> io::Result<()> {
    // Extract image paths if the `--style | -s custom` option is provided in the CLI.
    // - If the `custom` style is selected but no image path is provided, print an error and exit.
    // - Otherwise, if `custom` is selected, skip the first argument (which may be the style
    //   option) and collect the rest as image paths.
    // - If a different style is selected, use all provided arguments as they are.
    let args = if matches!(app.style, StyleOps::Custom | StyleOps::FromFile) && app.files.len() < 2
    {
        eprintln!("ERROR: Image Path Not Found");
        std::process::exit(1);
    } else if matches!(app.style, StyleOps::Custom | StyleOps::FromFile) {
        app.files.iter().skip(1).cloned().collect()
    } else {
        app.files.clone()
    };
    for ref path in args {
        let img = ImageReader::open(path)?.decode().unwrap_or_else(|err| {
            eprintln!("{}", err);
            std::process::exit(1);
        });
        let filter = FilterType::CatmullRom;
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
        let output_type = match path.extension() {
            Some(v) if v == "html" => OutputType::html(),
            Some(v) if v == "svg" => OutputType::svg(),
            _ => OutputType::term(),
        };
        match (&app.style, &app.colored) {
            (StyleOps::Ascii, true) => {
                let pi = PixtImg::new(ImgStyle::Ascii, output_type.color(ColorType::AvgFgOnly));
                pi.print(&img, &mut stdout)?;
            }
            (StyleOps::Ascii, false) => {
                let pi = PixtImg::new(ImgStyle::Ascii, output_type.color(ColorType::None));
                pi.print(&img, &mut stdout)?;
            }
            (StyleOps::Block, true) => {
                let pi = PixtImg::new(ImgStyle::Block, output_type.color(ColorType::AvgFgOnly));
                pi.print(&img, &mut stdout)?;
            }
            (StyleOps::Block, false) => {
                let pi = PixtImg::new(ImgStyle::Block, output_type.color(ColorType::None));
                pi.print(&img, &mut stdout)?;
            }
            (StyleOps::Pixel, true) => {
                let pi = PixtImg::new(ImgStyle::Pixel, output_type.color(ColorType::FgTopBgDown));
                pi.print(&img, &mut stdout)?;
            }
            (StyleOps::Pixel, false) => {
                let pi = PixtImg::new(ImgStyle::Pixel, output_type.color(ColorType::None));
                pi.print(&img, &mut stdout)?;
            }
            (StyleOps::Braills, true) => {
                let pi = PixtImg::new(ImgStyle::Braills, output_type.color(ColorType::AvgFgOnly));
                pi.print(&img, &mut stdout)?;
            }
            (StyleOps::Braills, false) => {
                let pi = PixtImg::new(ImgStyle::Braills, output_type.color(ColorType::None));
                pi.print(&img, &mut stdout)?;
            }
            (StyleOps::Dots, true) => {
                let pi = PixtImg::new(ImgStyle::Dots, output_type.color(ColorType::AvgFgOnly));
                pi.print(&img, &mut stdout)?;
            }
            (StyleOps::Dots, false) => {
                let pi = PixtImg::new(ImgStyle::Dots, output_type.color(ColorType::None));
                pi.print(&img, &mut stdout)?;
            }
            (StyleOps::Custom, false) => {
                let input = app.files[0]
                    .clone()
                    .into_os_string()
                    .into_string()
                    .unwrap_or_else(|err| {
                        eprintln!("ERROR: envalid chars: '{:?}'", err);
                        std::process::exit(1)
                    })
                    .chars()
                    .collect::<Vec<char>>();
                let pi = PixtImg::new(input, output_type.color(ColorType::None));
                pi.print(&img, &mut stdout)?;
            }
            (StyleOps::Custom, true) => {
                let input = app.files[0]
                    .clone()
                    .into_os_string()
                    .into_string()
                    .unwrap_or_else(|err| {
                        eprintln!("ERROR: envalid chars: '{:?}'", err);
                        std::process::exit(1)
                    })
                    .chars()
                    .collect::<Vec<char>>();
                let pi = PixtImg::new(input, output_type.color(ColorType::AvgFgOnly));
                pi.print(&img, &mut stdout)?;
            }
            (StyleOps::FromFile, _) => {
                let path = app.files[0]
                    .clone()
                    .into_os_string()
                    .into_string()
                    .unwrap_or_else(|err| {
                        eprintln!("ERROR: envalid chars: '{:?}'", err);
                        std::process::exit(1);
                    });
                let file = fs::File::open(path).unwrap_or_else(|err| {
                    eprintln!("{}", err);
                    std::process::exit(1)
                });
                let mut reader = BufReader::new(file);
                let mut val = String::new();
                reader.read_to_string(&mut val)?;
                let data = val
                    .lines()
                    .map(|v| v.trim().chars().collect())
                    .filter(|v: &Vec<char>| !v.is_empty())
                    .collect::<Vec<Vec<char>>>();

                let pi = PixtImg::new(data, output_type.color(ColorType::AvgFgOnly));
                pi.print(&img, &mut stdout)?;
            }
        }
    }
    Ok(())
}
