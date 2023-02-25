use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::{Not, Range};
use crate::pieces::{BOTTOM_ROW, FIRST_SQUARE, Pieces, TOP_ROW};
use crate::util::{Board, Directions, GameStatus, Position, Teams};

pub mod attacks;
pub mod notation;
pub mod pieces;
pub mod util;

const STARTING_BOARD: [u64; 6 * 2 + 1] = [
    0b00000000_11111111_00000000_00000000_00000000_00000000_00000000_00000000u64, //Black Pawns
    0b01000010_00000000_00000000_00000000_00000000_00000000_00000000_00000000u64, //Black Knights
    0b00100100_00000000_00000000_00000000_00000000_00000000_00000000_00000000u64, //Black Bishops
    0b10000001_00000000_00000000_00000000_00000000_00000000_00000000_00000000u64, //Black Rooks
    0b00010000_00000000_00000000_00000000_00000000_00000000_00000000_00000000u64, //Black King
    0b00001000_00000000_00000000_00000000_00000000_00000000_00000000_00000000u64, //Black Queen
    0b00000000_00000000_00000000_00000000_00000000_00000000_11111111_00000000u64, //White Pawns
    0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01000010u64, //White Knights
    0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00100100u64, //White Bishops
    0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_10000001u64, //White Rooks
    0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00010000u64, //White King
    0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00001000u64, //White Queen
    0xFF_FF_00_00_00_00_FF_FF //All pieces
];

pub const PIECES: [char; 12] = ['♟', '♞', '♝', '♜', '♛', '♚', '♙', '♘', '♗', '♖', '♕', '♔'];

#[derive(Clone)]
pub struct ChessBoard {
    pub board: Board,
    //The move number, starting at 0, evens are white moves, odds are black moves (move_number % team)
    pub move_number: u16,
    //Counter counting down from 100. Resets on capture or pawn move
    pub fifty_move_counter: u8,
    //Whether different sides are disqualified from castling. Still may not be possible due to check.
    pub castle_status: [bool; 4],
    //Location of the last moved pawn for en passant
    pub last_pawn: Option<Position>,
    //Status of the game
    pub game_status: GameStatus,
    //Last board states since significant move (for repetition draw) and how often they've happened
    pub significant_boards: HashMap<Board, u8>,
}

impl ChessBoard {
    pub fn new() -> Self {
        return Self {
            board: Board(STARTING_BOARD.clone()),
            move_number: 0,
            fifty_move_counter: 100,
            castle_status: [true; 4],
            last_pawn: None,
            game_status: GameStatus::Ongoing,
            significant_boards: HashMap::new(),
        };
    }

    pub fn move_piece(&mut self, piece: Pieces, team: Teams, location: Position, target: Position, promotion: Option<Pieces>) -> bool {
        //Make sure the piece exists
        if (self.board[piece as usize + team as usize] & location) == 0 {
            return false;
        }

        //Make sure the move is a legal move
        if !piece.get_moves(location, team, &self, false).contains(&target) {
            return false;
        }

        //Look for a target to take on the enemy team
        for piece in (!team).pieces() {
            if target.is_occupied_by(&self.board, piece, team) {
                self.board[12] ^= target;
                self.board[piece] ^= target;
                break;
            }
        }

        //Move it on the global board
        self.board[12] = (self.board[12] ^ location) + target;

        //Pawn promotion rules
        if let Pieces::Pawn = piece {
            let promotion_squares = match &team {
                Teams::White => TOP_ROW,
                Teams::Black => BOTTOM_ROW
            };
            if location.on_row(promotion_squares) {
                return match promotion {
                    Some(promoting) => if promoting.promotable_into() {
                        self.board[promoting as usize + team as usize] += target;
                        self.board[piece as usize + team as usize] ^= location;
                        self.last_pawn = Some(location);
                        self.check_game_status();
                        true
                    } else {
                        false
                    }
                    None => false
                };
            } else {
                self.last_pawn = Some(location);
            }
        } else {
            self.last_pawn = None;
        }

        //TODO castling

        //Move the piece
        let piece = piece as usize + team as usize;
        self.board[piece] = (self.board[piece] ^ location) + target;
        self.check_game_status();
        return true;
    }

    fn check_game_status(&mut self) {
        self.move_number += 1;
        if self.fifty_move_counter == 0 {
            self.game_status = GameStatus::DrawByFiftyMoveRule;
            return;
        }
        self.fifty_move_counter -= 1;

        match self.significant_boards.get_mut(&self.board[0..12]) {
            Some(count) => {
                if count == &3 {
                    self.game_status = GameStatus::DrawByRepetition;
                    return;
                }
                *count -= 1;
            }
            None => {
                let mut adding = [0u64; 12];
                adding.clone_from_slice(&self.board[0..12]);
                self.significant_boards.insert(adding, 1);
            }
        }

        self.game_status = self.checkmate_or_stalemate(Teams::White);

        if let GameStatus::Ongoing = self.game_status {
            self.game_status = self.checkmate_or_stalemate(Teams::Black);
        }
    }

    fn checkmate_or_stalemate(&mut self, team: Teams) -> GameStatus {
        //If the king can move, it's automatically not over
        if !Pieces::King.get_moves(self.board[Pieces::King as usize], Teams::White, &self, false).is_empty() {
            return GameStatus::Ongoing;
        }

        let win_if_no_moves = is_in_check(&self, self.board[Pieces::King as usize + team as usize], team);

        //Check to make sure it's not stalemate
        for i in team.pieces() {
            let piece_type = Pieces::from(i as u8);
            for piece in self.get_pieces(i) {
                if !piece_type.get_moves(piece, team, self, false).is_empty() {
                    return GameStatus::Ongoing;
                }
            }
        }

        return if win_if_no_moves {
            (!team).win_status()
        } else {
            GameStatus::DrawByStalemate
        }
    }

    fn get_pieces(&self, piece: usize) -> Vec<Position> {
        let mut output = Vec::new();

        let mut index = 1u64;
        for _ in 0..64 {
            if (self.board[piece] & index) != 0 {
                output.push(Position(index));
            }
            index <<= 1;
        }
        return output;
    }
}

//Color codes are 10 bytes, chars are 1 each
impl Display for ChessBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut board = String::new();

        let mut index = 0b1u64;
        for i in 0..64 {
            let mut found = false;
            for j in 0..12 {
                if self.board[j] & index != 0 {
                    found = true;
                    board.push(PIECES[j]);
                    break;
                }
            }

            if !found {
                board.push('\u{2002}');
                board.push('\u{2002}');
            }

            if i % 8 == 7 {
                board.push('\n');
            }
            index = index << 1;
        }
        return write!(f, "{}", board);
    }
}