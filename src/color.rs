#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Color
{
    White,
    Black
}

#[inline]
pub fn opposite_color(color: Color) -> Color {
    match color {
        Color::White => Color::Black,
        Color::Black => Color::White
    }
}
