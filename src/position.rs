use regex::Captures;
use regex::Regex;

use color::Color;
use color::opposite_color;
use piece::PieceType;


const BOARD_NUM_CELLS: usize = 120;

const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

const ROW_1: usize = 7;
const ROW_2: usize = 6;
const ROW_3: usize = 5;
const ROW_4: usize = 4;
const ROW_5: usize = 3;
const ROW_6: usize = 2;
const ROW_7: usize = 1;
const ROW_8: usize = 0;

const COL_A: usize = 0;
const COL_B: usize = 1;
const COL_C: usize = 2;
const COL_D: usize = 3;
const COL_E: usize = 4;
const COL_F: usize = 5;
const COL_G: usize = 6;
const COL_H: usize = 7;


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cell {
    Empty,
    Piece(PieceType, Color)
}

#[derive(Debug, PartialEq)]
pub struct Position {
    cells: [[Cell; 8]; 8],
    side_to_move: Color,
    halfmove_clock: u32,
    white_can_castle_kingside: bool,
    white_can_castle_queenside: bool,
    black_can_castle_kingside: bool,
    black_can_castle_queenside: bool,
    en_passant: Option<(usize, usize)>
}

impl Position {
    pub fn new() -> Position {
        Position::from_fen(STARTPOS_FEN)
    }

    // TODO: Panics if we're given invalid FEN.
    // That would mean either we've got a bug or the GUI is not sane anyhow,
    // so probably no big loss.
    // NOTE: move number will be ignored because it's not interesting.
    pub fn from_fen(fen: &str) -> Position {
        lazy_static! {
            static ref RE: Regex = Regex::new("^([PpNnBbRrQqKk1-8]+)/\
                                               ([PpNnBbRrQqKk1-8]+)/\
                                               ([PpNnBbRrQqKk1-8]+)/\
                                               ([PpNnBbRrQqKk1-8]+)/\
                                               ([PpNnBbRrQqKk1-8]+)/\
                                               ([PpNnBbRrQqKk1-8]+)/\
                                               ([PpNnBbRrQqKk1-8]+)/\
                                               ([PpNnBbRrQqKk1-8]+) \
                                               (?P<side_to_move>[wb]) \
                                               (?P<castling>[KQkq]+|-) \
                                               (?P<en_passant>[a-h][1-8]|-) \
                                               (?P<halfmove_clock>[0-9]+) \
                                               (?P<move_number>[0-9]+)$").unwrap();
        }
        let captures = RE.captures(fen);
        if captures.is_none() {
            panic!("Invalid FEN (failed to match regex)");
        }
        let captures = captures.unwrap();
        let side_to_move = match captures.name("side_to_move").unwrap().as_str() {
            "w" => Color::White,
            "b" => Color::Black,
            _ => panic!("Can't get here (invalid color)")
        };
        let halfmove_clock = captures.name("halfmove_clock").unwrap().as_str().parse::<u32>().unwrap();
        let castling = captures.name("castling").unwrap().as_str();
        let en_passant = captures.name("en_passant").unwrap().as_str();
        let en_passant = match en_passant {
            "-" => None,
            _ => Some(parse_algebraic_coords(en_passant))
        };
        Position {
            cells: read_board_data(&captures),
            side_to_move: side_to_move,
            halfmove_clock: halfmove_clock,
            white_can_castle_kingside: castling.contains('K'),
            white_can_castle_queenside: castling.contains('Q'),
            black_can_castle_kingside: castling.contains('k'),
            black_can_castle_queenside: castling.contains('q'),
            en_passant: en_passant
        }
    }

    // NOTE: Does not check move's legality! It just replaces the destination square with the
    // contents of the source square. However, it does recognize castling (but doesn't check the
    // legality of it) and en passant captures (ditto). It will also adjust castling rights, the en
    // passant square, the half-move clock, and whose turn it is to move. This should be sufficient
    // for handling the "moves" subcommand of the "position" command, as well as the result of move
    // generation for the AI.
    //
    // In addition to keeping the code simpler, it also keeps the AI code fast.
    pub fn make_move(&mut self,
                     src: (usize, usize),
                     dst: (usize, usize),
                     promotion_type: Option<PieceType>) {
        let (src_row, src_col) = src;
        let (dst_row, dst_col) = dst;
        let src_cell = self.cells[src_row][src_col];
        self.cells[src_row][src_col] = Cell::Empty;
        self.halfmove_clock = if self.cells[dst_row][dst_col] == Cell::Empty {
            // Not a capture; advance half-move clock
            self.halfmove_clock + 1
        } else {
            // Capture; reset half-move clock
            0
        };
        self.cells[dst_row][dst_col] = src_cell;
        if let Cell::Piece(piece_type, color) = src_cell {
            match piece_type {
                PieceType::Pawn => {
                    self.halfmove_clock = 0;                // Pawn moves reset half-move clock
                    if let Some((ep_row, ep_col)) = self.en_passant {
                        // An en passant capture may be possible
                        if (dst_row, dst_col) == (ep_row, ep_col) {
                            // Capturing en passant; remove the captured pawn
                            // Notice the captured pawn is on the *source* row and on the
                            // *destination* column
                            self.cells[src_row][dst_col] = Cell::Empty;
                        }
                    }
                    if dst_row == ROW_1 || dst_row == ROW_8 {
                        // Promote piece
                        // TODO: Better error handling if we can't unwrap
                        self.cells[dst_row][dst_col] = Cell::Piece(promotion_type.unwrap(),
                                                                   color);
                    }
                },
                PieceType::Rook => {
                    if src_row == ROW_1 {
                        if src_col == COL_A {
                            self.white_can_castle_queenside = false;
                        } else if src_col == COL_H {
                            self.white_can_castle_kingside = false;
                        }
                    } else if src_row == ROW_8 {
                        if src_col == COL_A {
                            self.black_can_castle_queenside = false;
                        } else if src_col == COL_H {
                            self.black_can_castle_kingside = false;
                        }
                    }
                },
                PieceType::King => {
                    match color {
                        Color::White => {
                            self.white_can_castle_kingside = false;
                            self.white_can_castle_queenside = false;
                        },
                        Color::Black => {
                            self.black_can_castle_kingside = false;
                            self.black_can_castle_queenside = false;
                        }
                    }
                    if src_col == COL_E {
                        if dst_col == COL_G {
                            // Kingside castling; move the rook too
                            let rook = self.cells[src_row][COL_H];
                            self.cells[src_row][COL_H] = Cell::Empty;
                            self.cells[src_row][COL_F] = rook;
                        }
                        else if src_col == COL_E && dst_col == COL_C {
                            // Queenside castling; move the rook too
                            let rook = self.cells[src_row][COL_A];
                            self.cells[src_row][COL_A] = Cell::Empty;
                            self.cells[src_row][COL_D] = rook;
                        }
                    }
                }
                _ => ()
            }
            self.en_passant = {
                if piece_type == PieceType::Pawn {
                    if src_row == ROW_2 && dst_row == ROW_4 {
                        // White is making a two-square pawn thrust
                        Some((dst_row + 1, dst_col))
                    } else if src_row == ROW_7 && dst_row == ROW_5 {
                        // Black is making a two-square pawn thrust
                        Some((dst_row - 1, dst_col))
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
        } else {
            panic!("Move from empty square");
        }
        self.side_to_move = opposite_color(self.side_to_move);
    }
}

fn read_board_data(captures: &Captures) -> [[Cell; 8]; 8] {
    let mut cells: [[Cell; 8]; 8] = [[Cell::Empty; 8]; 8];
    for row in 0..8 {
        let row_str = &captures[row+1];
        let mut col = 0usize;
        for ch in row_str.chars() {
            if let Some(digit) = ch.to_digit(10) {
                col += digit as usize;
            } else {
                cells[row][col] = match ch {
                    'P' => Cell::Piece(PieceType::Pawn, Color::White),
                    'p' => Cell::Piece(PieceType::Pawn, Color::Black),
                    'N' => Cell::Piece(PieceType::Knight, Color::White),
                    'n' => Cell::Piece(PieceType::Knight, Color::Black),
                    'B' => Cell::Piece(PieceType::Bishop, Color::White),
                    'b' => Cell::Piece(PieceType::Bishop, Color::Black),
                    'R' => Cell::Piece(PieceType::Rook, Color::White),
                    'r' => Cell::Piece(PieceType::Rook, Color::Black),
                    'Q' => Cell::Piece(PieceType::Queen, Color::White),
                    'q' => Cell::Piece(PieceType::Queen, Color::Black),
                    'K' => Cell::Piece(PieceType::King, Color::White),
                    'k' => Cell::Piece(PieceType::King, Color::Black),
                    _ => panic!("Can't get here (invalid piece type: {})", ch)
                };
                col += 1;
            }
        }
    }
    cells
}

// This should probably be moved into a move generator module
pub fn can_piece_move_into_cell(cell: Cell, my_color: Color) -> bool {
    match cell {
        Cell::Empty => true,
        Cell::Piece(_, c) if c == opposite_color(my_color) => true,
        _ => false
    }
}

// NOTE: Does not check the validity of the string!
fn parse_algebraic_coords(coords: &str) -> (usize, usize) {
    let mut iter = coords.chars();
    let col = iter.next().unwrap() as usize - 'a' as usize;
    let row = 7 - (iter.next().unwrap() as usize - '1' as usize);
    (row, col)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_equals_startpos() {
        assert_eq!(Position::new(), Position::from_fen(STARTPOS_FEN));
    }

    #[test]
    fn moves_1e4() {
        let mut pos = Position::new();
        pos.make_move((ROW_2, COL_E), (ROW_4, COL_E), None);
        assert_eq!(pos,
                   Position::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"));
    }

    #[test]
    fn giuoco_piano() {
        let mut pos = Position::new();
        pos.make_move((ROW_2, COL_E), (ROW_4, COL_E), None);    // 1.e4
        pos.make_move((ROW_7, COL_E), (ROW_5, COL_E), None);    // 1...e5
        pos.make_move((ROW_1, COL_G), (ROW_3, COL_F), None);    // 2.Nf3
        pos.make_move((ROW_8, COL_B), (ROW_6, COL_C), None);    // 2...Nc6
        pos.make_move((ROW_1, COL_F), (ROW_4, COL_C), None);    // 3.Bc4
        pos.make_move((ROW_8, COL_F), (ROW_5, COL_C), None);    // 3.Bc5
        assert_eq!(pos,
                   Position::from_fen("r1bqk1nr/pppp1ppp/2n5/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4"));
    }

    #[test]
    fn white_capturing_en_passant() {
        let mut pos = Position::from_fen("k7/p7/8/1P6/8/8/8/K7 b - - 0 1");
        pos.make_move((ROW_7, COL_A), (ROW_5, COL_A), None);    // 1...a5
        pos.make_move((ROW_5, COL_B), (ROW_6, COL_A), None);    // 2.bxa4 e.p.
        assert_eq!(pos,
                   Position::from_fen("k7/8/P7/8/8/8/8/K7 b - - 0 2"));
    }

    #[test]
    fn black_capturing_en_passant() {
        let mut pos = Position::from_fen("k7/8/8/8/1p6/8/P7/K7 w - - 0 1");
        pos.make_move((ROW_2, COL_A), (ROW_4, COL_A), None);    // 1.a4
        pos.make_move((ROW_4, COL_B), (ROW_3, COL_A), None);    // 1...bxa3 e.p.
        assert_eq!(pos,
                   Position::from_fen("k7/8/8/8/8/p7/8/K7 w - - 0 2"));
    }

    #[test]
    fn not_en_passant() {
        // Looks like en passant, but isn't, so don't remove the black pawn on row 6
        let mut pos = Position::from_fen("k7/4p3/8/3Pp3/8/8/8/K7 b - - 0 1");
        pos.make_move((ROW_7, COL_E), (ROW_6, COL_E), None);    // 1...e6
        pos.make_move((ROW_5, COL_D), (ROW_6, COL_E), None);    // 2.dxe6
        assert_eq!(pos,
                   Position::from_fen("k7/8/4P3/4p3/8/8/8/K7 b - - 0 2"));
    }

    #[test]
    fn white_castling_kingside() {
        let mut pos = Position::from_fen("k7/8/8/8/8/8/8/4K2R w K - 0 1");
        pos.make_move((ROW_1, COL_E), (ROW_1, COL_G), None);    // 1.O-O
        assert_eq!(pos,
                   Position::from_fen("k7/8/8/8/8/8/8/5RK1 b - - 1 1"));
    }

    #[test]
    fn black_castling_kingside() {
        let mut pos = Position::from_fen("4k2r/8/8/8/8/8/8/K7 b k - 0 1");
        pos.make_move((ROW_8, COL_E), (ROW_8, COL_G), None);    // 1...O-O
        assert_eq!(pos,
                   Position::from_fen("5rk1/8/8/8/8/8/8/K7 w - - 1 2"));
    }

    #[test]
    fn white_castling_queenside() {
        let mut pos = Position::from_fen("k7/8/8/8/8/8/8/R3K3 w K - 0 1");
        pos.make_move((ROW_1, COL_E), (ROW_1, COL_C), None);    // 1.O-O-O
        assert_eq!(pos,
                   Position::from_fen("k7/8/8/8/8/8/8/2KR4 b - - 1 1"));
    }

    #[test]
    fn black_castling_queenside() {
        let mut pos = Position::from_fen("r3k3/8/8/8/8/8/8/K7 b k - 0 1");
        pos.make_move((ROW_8, COL_E), (ROW_8, COL_C), None);    // 1...O-O-O
        assert_eq!(pos,
                   Position::from_fen("2kr4/8/8/8/8/8/8/K7 w - - 1 2"));
    }

    #[test]
    fn promote_pawn_to_knight() {
        let mut pos = Position::from_fen("k7/4P3/8/8/8/8/8/K7 w - - 0 1");
        pos.make_move((ROW_7, COL_E), (ROW_8, COL_E), Some(PieceType::Knight)); // 1.e8=N
        assert_eq!(pos,
                   Position::from_fen("k3N3/8/8/8/8/8/8/K7 b - - 0 1"));
    }
}
