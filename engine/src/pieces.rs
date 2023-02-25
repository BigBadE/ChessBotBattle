use std::fmt::{Display, Formatter};
use crate::ChessBoard;
use crate::util::{PiecePositions, Position, Row, Teams};

pub const FIRST_SQUARE: u64 = 0b1;
pub const TOP_ROW: Row = Row(0xFF_00_00_00_00_00_00_00);
pub const SECOND_ROW: Row = Row(0x00_FF_00_00_00_00_00_00);
pub const SEVENTH_ROW: Row = Row(0x00_00_00_00_00_00_FF_00);
pub const BOTTOM_ROW: Row = Row(0x00_00_00_00_00_00_00_FF);
pub const LEFT_SIDE: Row = Row(0x90_90_90_90_90_90_90_90);
pub const RIGHT_SIDE: Row = Row(0x01_01_01_01_01_01_01_01);

#[derive(Clone, Copy)]
pub enum Pieces {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl Pieces {
    pub fn get_moves(&self, position: Position, team: Teams, board: &ChessBoard, ignore_check: bool) -> Vec<Position> {
        let possible = match self {
            Pieces::Pawn => match team {
                Teams::White => {
                    let mut found = Vec::new();
                    if board.board[12] & (position >> 8) == 0 {
                        found.push(position >> 8);

                        //Double move on the first row
                        if position & SECOND_ROW != 0 && board.board[12] & (position >> 16) == 0 {
                            found.push(position >> 16);
                        }
                    }

                    //Check if the pawn can take left
                    if board.board[12] & (position >> 7) != 0 {
                        let mut can_take = false;
                        for piece in (!team).pieces() {
                            if board.board[piece] & (position >> 7) != 0 {
                                can_take = true;
                            }
                        }
                        if can_take {
                            found.push(position >> 7);
                        }
                    }

                    //Check if pawn can take right
                    if board.board[12] & (position >> 9) != 0 {
                        let mut can_take = false;
                        for piece in (!team).pieces() {
                            if board.board[piece] & (position >> 9) != 0 {
                                can_take = true;
                            }
                        }
                        if can_take {
                            found.push(position >> 9);
                        }
                    }

                    //En passant. Make sure a pawn move was last, it's the enemy team, and the pawn is beside it.
                    if board.last_pawn.is_some() &&
                        board.board[!team as usize + Pieces::Pawn as usize] & board.last_pawn.unwrap() != 0 &&
                        (board.last_pawn.unwrap() == (position >> 1) || board.last_pawn.unwrap() == (position << 1)) {
                        found.push(board.last_pawn.unwrap() >> 9);
                    }
                    found
                }
                Teams::Black => {
                    let mut found = Vec::new();
                    if board.board[12] & (position << 8) == 0 {
                        found.push(position << 8);

                        //Double move on the first row
                        if position & SEVENTH_ROW != 0 && board.board[12] & (position << 16) == 0 {
                            found.push(position << 16);
                        }
                    }

                    //Check if the pawn can take left
                    if board.board[12] & (position << 7) != 0 {
                        let mut can_take = false;
                        for piece in (!team).pieces() {
                            if board.board[piece] & (position << 7) != 0 {
                                can_take = true;
                            }
                        }
                        if can_take {
                            found.push(position << 7);
                        }
                    }

                    //Check if pawn can take right
                    if board.board[12] & (position << 9) != 0 {
                        let mut can_take = false;
                        for piece in (!team).pieces() {
                            if board.board[piece] & (position << 9) != 0 {
                                can_take = true;
                            }
                        }
                        if can_take {
                            found.push(position << 9);
                        }
                    }

                    //En passant. Make sure a pawn move was last, it's the enemy team, and the pawn is beside it.
                    if board.last_pawn.is_some() &&
                        board.board[!team as usize + Pieces::Pawn as usize] & board.last_pawn.unwrap() != 0 &&
                        (board.last_pawn.unwrap() == (position >> 1) || board.last_pawn.unwrap() == (position << 1)) {
                        found.push(board.last_pawn.unwrap() << 9);
                    }
                    found
                }
            },
            Pieces::Bishop => {
                let mut output = Vec::new();
                //Down Right
                Self::move_piece(&mut output, position, team, board,
                                 |position| position & BOTTOM_ROW != 0 || position & RIGHT_SIDE != 0,
                                 |position| position >> 9);
                //Down Left
                Self::move_piece(&mut output, position, team, board,
                                 |position| position & BOTTOM_ROW != 0 || position & LEFT_SIDE != 0,
                                 |position| position >> 7);
                //Up Right
                Self::move_piece(&mut output, position, team, board,
                                 |position| position & TOP_ROW != 0 || position & LEFT_SIDE != 0,
                                 |position| -> u64 { position << 9 });
                //Up Left
                Self::move_piece(&mut output, position, team, board,
                                 |position| position & TOP_ROW != 0 || position & RIGHT_SIDE != 0,
                                 |position| -> u64 { position << 7 });
                output
            }
            Pieces::Rook => {
                let mut output = Vec::new();
                //Down Right
                Self::move_piece(&mut output, position, team, board, |position| position & BOTTOM_ROW != 0,
                                 |position| position >> 8);
                //Down Left
                Self::move_piece(&mut output, position, team, board, |position| position & RIGHT_SIDE != 0,
                                 |position| position >> 1);
                //Up Right
                Self::move_piece(&mut output, position, team, board, |position| position & TOP_ROW != 0,
                                 |position| -> u64 { position << 8 });
                //Up Left
                Self::move_piece(&mut output, position, team, board, |position| position & LEFT_SIDE != 0,
                                 |position| -> u64 { position << 1 });
                output
            }
            Pieces::Queen => {
                let mut output = Pieces::Bishop.get_moves(position, team, board, ignore_check);
                output.append(&mut Pieces::Rook.get_moves(position, team, board, ignore_check));
                output
            }
            Pieces::Knight => {
                let mut possible = Vec::new();
                if position & BOTTOM_ROW != 0 {
                    possible.push(position >> 17);
                    possible.push(position >> 15);
                }
                if position & (BOTTOM_ROW + SEVENTH_ROW) != 0 {
                    possible.push(position >> 10);
                    possible.push(position >> 6);
                }
                if position & (TOP_ROW) != 0 {
                    possible.push(position << 10);
                    possible.push(position << 6);
                }
                if position & (TOP_ROW + SECOND_ROW) != 0 {
                    possible.push(position << 10);
                    possible.push(position << 6);
                }
                let mut output = Vec::new();
                for position in possible {
                    for piece in team.pieces() {
                        if board.board[piece] & position == 0 {
                            output.push(position);
                        }
                    }
                }
                output
            }
            Pieces::King => {
                let possible = [position << 9, position << 8, position << 7, position << 1, position >> 1, position >> 7, position >> 8, position >> 9];
                let mut output = Vec::new();
                for position in possible {
                    for piece in team.pieces() {
                        if board.board[piece] & position == 0 {
                            output.push(position);
                        }
                    }
                }
                output
            }
        };

        if ignore_check {
            return possible;
        }

        let mut output = Vec::new();
        for checking in &possible {
            if !creates_check(board.clone(), team, position, *checking) {
                output.push(*checking);
            }
        }
        return output;
    }

    //Move a sliding piece according to the function with respect to the border
    fn move_piece(output: &mut Vec<Position>, position: Position, team: Teams, board: &ChessBoard, direction: Directions) {
        let mut position = position.clone();
        loop {
            position = direction.offset(position);
            if position == 0 {
                break;
            }

            //Check global board
            if position.is_occupied(&board) {
                //Check enemy pieces
                if position.occupied_by_team(&board, !team) {
                    continue;
                }
                break;
            }
            output.push(position);
        }
    }

    pub fn promotable_into(&self) -> bool {
        match self {
            Pieces::Pawn => false,
            Pieces::Bishop => true,
            Pieces::Knight => true,
            Pieces::Rook => true,
            Pieces::Queen => true,
            Pieces::King => false
        }
    }
}

impl From<u8> for Pieces {
    fn from(value: u8) -> Self {
        if value > 12 {
            panic!("Too big of a piece!");
        }

        return match value % 6 {
            0 => Pieces::Pawn,
            1 => Pieces::Knight,
            2 => Pieces::Bishop,
            3 => Pieces::Rook,
            4 => Pieces::King,
            5 => Pieces::Queen,
            _ => panic!("Wtf?!?")
        };
    }
}

impl Display for Pieces {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return match self {
            Pieces::Pawn => write!(f, "Pawn"),
            Pieces::Rook => write!(f, "Rook"),
            Pieces::Knight => write!(f, "Knight"),
            Pieces::Bishop => write!(f, "Bishop"),
            Pieces::King => write!(f, "King"),
            Pieces::Queen => write!(f, "Queen")
        };
    }
}

impl From<char> for Pieces {
    fn from(character: char) -> Self {
        return match character {
            'Q' => Pieces::Queen,
            'N' => Pieces::Knight,
            'B' => Pieces::Bishop,
            'R' => Pieces::Rook,
            'K' => Pieces::Rook,
            _ => Pieces::Pawn
        };
    }
}