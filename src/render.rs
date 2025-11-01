use std::io::{self, Write};

use image::DynamicImage;

use crate::img::PixtImg;

pub fn render(p: &PixtImg, img: &DynamicImage, out: impl Write) -> io::Result<()> {
    p.print(img, out)
}
