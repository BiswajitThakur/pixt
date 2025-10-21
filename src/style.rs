use crate::img::IntoPixtData;

pub enum ImgStyle {
    Ascii,
    Block,
    Pixel,
    Braills,
    Dots,
}

impl IntoPixtData for ImgStyle {
    fn into_pixt_data(self) -> Vec<Vec<char>> {
        match self {
            Self::Ascii => [' ', '.', '-', '~', '+', '*', '%', '#', '@'].into_pixt_data(),
            Self::Block => [' ', '░', '▒', '▓'].into_pixt_data(),
            Self::Pixel => [' ', '▀', '▞', '▟', '█'].into_pixt_data(),
            Self::Braills => [
                [' ', '⠁', '⠉', '⠓', '⠛'],
                ['⠄', '⠅', '⠩', '⠝', '⠟'],
                ['⠤', '⠥', '⠭', '⠯', '⠽'],
                ['⠴', '⠵', '⠽', '⠾', '⠿'],
                ['⠶', '⠾', '⠾', '⠿', '⠿'],
            ]
            .into_pixt_data(),
            Self::Dots => [' ', '⠂', '⠒', '⠕', '⠞', '⠟', '⠿'].into_pixt_data(),
        }
    }
}
