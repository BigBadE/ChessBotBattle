use std::fmt::{Display, Formatter};
use std::ops::{Not, Range};
use crate::ChessBoard;
use crate::pieces::{FIRST_SQUARE, Pieces};

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Position(pub u64);

pub struct PiecePositions(pub u64);

pub struct Row(pub u64);

//A bitboard representation of the board, with each piece for each team having a binary board.
pub struct Board(pub [u64; 13]);

impl From<&str> for Position {
    #[inline]
    fn from(pos: &str) -> Self {
        return Position::from((pos.as_bytes()[0] - b'a', pos.as_bytes()[1] - b'1'))
    }
}

impl From<(u8, u8)> for Position {
    fn from((x, y): (u8, u8)) -> Self {
        return Position(0b1 << ((x * 8) + y));
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut index = self.0.clone();
        for i in 0..63 {
            if index == 1 {
                return write!(f, "{}{}", (b'a' + i % 8) as char, (b'1' + i / 8) as char);
            }

            index >>= 1;
        }
        panic!("Wtf");
    }
}

impl Position {
    #[inline]
    pub fn on_row(&self, row: Row) -> bool {
        return self.0 & row.0 != 0;
    }

    #[inline]
    pub fn is_occupied(&self, board: &Board) -> bool {
        return board.0[12] & self.0 != 0;
    }

    #[inline]
    pub fn is_occupied_by(&self, board: &Board, piece: Pieces, team: Teams) -> bool {
        return board.0[piece as usize + team as usize] != 0;
    }

    #[inline]
    pub fn occupied_by_team(&self, board: &Board, team: Teams) -> bool {
        for piece in team.pieces() {
            if board.0[piece as usize + team as usize] != 0 {
                return true;
            }
        }
        return false;
    }
}

impl Board {
    #[inline]
    pub fn get_board(&self, piece: Pieces, team: Teams) -> PiecePositions {
        return PiecePositions(self.0[team as usize + piece as usize]);
    }

    #[inline]
    pub fn move_piece(&self, position: Position, target: Position, team: Teams) {

    }
}

fn is_in_check(board: &ChessBoard, position: &Position, team: Teams) -> bool {
    for piece in (!team).non_king_pieces() {
        let piece_enum = Pieces::from(piece as u8);
        let mut index = Position(FIRST_SQUARE);
        for _ in 0..64 {
            if index.is_occupied_by(&board.board, piece_enum, !team) &&
                piece_enum.get_moves(index, !team, board, true).contains(position) {
                return true;
            }
            index = Directions::East.offset(index);
        }
    }
    return false;
}

#[derive(Clone)]
pub enum Directions {
    NorthWest = 7,
    North = 8,
    FarNorth = 16,
    NorthEast = 9,
    West = -1,
    FarWest = -2,
    East = 1,
    FarEast = 2,
    SouthWest = -9,
    South = -8,
    FarSouth = -16,
    SouthEast = -7
}

impl Directions {
    #[inline]
    pub fn offset(&self, position: Position) -> Position {
        let value = self.clone() as i8;
        return Position(if value < 0 {
            position.0 >> ((value * -1) as u8)
        } else {
            position.0 << value as u8
        });
    }
}

#[derive(Clone, Copy)]
pub enum Teams {
    White = 0,
    Black = 6,
}

impl Teams {
    pub fn win_status(&self) -> GameStatus {
        return match self {
            Teams::White => GameStatus::WhiteWin,
            Teams::Black => GameStatus::BlackWin
        };
    }

    pub fn non_king_pieces(&self) -> Range<usize> {
        return *self as usize..*self as usize + 5;
    }

    pub fn pieces(&self) -> Range<usize> {
        return *self as usize..*self as usize + 6;
    }
}

impl From<u16> for Teams {
    fn from(move_number: u16) -> Self {
        if move_number % 2 == 0 {
            return Teams::White;
        }
        return Teams::Black;
    }
}

impl Not for Teams {
    type Output = Teams;

    fn not(self) -> Teams {
        return match self {
            Teams::White => Teams::Black,
            Teams::Black => Teams::White
        };
    }
}

impl Display for Teams {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return match self {
            Teams::White => write!(f, "White"),
            Teams::Black => write!(f, "Black")
        }
    }
}

pub enum CastleDirection {
    Left = 0,
    Right = 1,
}

#[derive(Clone)]
pub enum GameStatus {
    Ongoing,
    BlackWin,
    WhiteWin,
    DrawByStalemate,
    DrawByInsufficientMaterial,
    DrawByRepetition,
    DrawByFiftyMoveRule,
}