use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::{Not, Range};

pub mod notation;

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

pub const TOP_ROW: u64 = 0xFF_00_00_00_00_00_00_00;
pub const SECOND_ROW: u64 = 0x00_FF_00_00_00_00_00_00;
pub const SEVENTH_ROW: u64 = 0x00_00_00_00_00_00_FF_00;
pub const BOTTOM_ROW: u64 = 0x00_00_00_00_00_00_00_FF;
pub const LEFT_SIDE: u64 = 0x90_90_90_90_90_90_90_90;
pub const RIGHT_SIDE: u64 = 0x01_01_01_01_01_01_01_01;
pub const PIECES: [char; 12] = ['♟', '♞', '♝', '♜', '♛', '♚', '♙', '♘', '♗', '♖', '♕', '♔'];

#[derive(Clone)]
pub struct ChessBoard {
    //A bitboard representation of the board, with each piece for each team having a binary board.
    pub board: [u64; 6 * 2 + 1],
    //The move number, starting at 0, evens are white moves, odds are black moves (move_number % team)
    pub move_number: u16,
    //Counter counting down from 100. Resets on capture or pawn move
    pub fifty_move_counter: u8,
    //Whether different sides are disqualified from castling. Still may not be possible due to check.
    pub castle_status: [bool; 4],
    //Location of the last moved pawn for en passant
    pub last_pawn: Option<u64>,
    //Status of the game
    pub game_status: GameStatus,
    //Last board states since significant move (for repetition draw) and how often they've happened
    pub significant_boards: HashMap<[u64; 12], u8>,
}

impl ChessBoard {
    pub fn new() -> Self {
        return Self {
            board: STARTING_BOARD.clone(),
            move_number: 0,
            fifty_move_counter: 100,
            castle_status: [true; 4],
            last_pawn: None,
            game_status: GameStatus::Ongoing,
            significant_boards: HashMap::new(),
        };
    }

    pub fn move_piece_bits(&mut self, piece: Pieces, team: Teams, location: u64, target: u64, promotion: Option<Pieces>) -> bool {
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
            if self.board[piece] & target != 0 {
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
            if location & promotion_squares != 0 {
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

    pub fn move_piece(&mut self, piece: Pieces, team: Teams, location: (u8, u8), target: (u8, u8), promotion: Option<Pieces>) -> bool {
        let target = (target.0, 7-target.1);
        let location = (location.0, 7-location.1);
        return self.move_piece_bits(piece, team, convert_to_flat(location), convert_to_flat(target), promotion);
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

    fn get_pieces(&self, piece: usize) -> Vec<u64> {
        let mut output = Vec::new();

        let mut index = 1u64;
        for _ in 0..64 {
            if (self.board[piece] & index) != 0 {
                output.push(index);
            }
            index <<= 1;
        }
        return output;
    }
}

pub fn convert_to_flat(location: (u8, u8)) -> u64 {
    return 0b1 << (location.0 + location.1 * 8);
}

fn creates_check(mut board: ChessBoard, team: Teams, location: u64, target: u64) -> bool {
    board.board[12] = board.board[12] ^ location + target;
    for i in (!team).pieces() {
        board.board[i] ^= target;
    }

    return is_in_check(&board, board.board[team as usize + Pieces::King as usize], team);
}

fn is_in_check(board: &ChessBoard, position: u64, team: Teams) -> bool {
    for piece in (!team).non_king_pieces() {
        let piece_enum = Pieces::from(piece as u8);
        let mut index = 1u64;
        for _ in 0..64 {
            if board.board[piece as usize] & index != 0 &&
                piece_enum.get_moves(index, !team, board, true).contains(&position) {
                return true;
            }
            index <<= 1;
        }
    }
    return false;
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

#[derive(Clone, Copy)]
pub enum Pieces {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5
}

impl Pieces {
    pub fn get_moves(&self, position: u64, team: Teams, board: &ChessBoard, ignore_check: bool) -> Vec<u64> {
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
            },
            Pieces::Queen => {
                let mut output = Pieces::Bishop.get_moves(position, team, board, ignore_check);
                output.append(&mut Pieces::Rook.get_moves(position, team, board, ignore_check));
                output
            },
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
            },
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
    fn move_piece(output: &mut Vec<u64>, position: u64, team: Teams, board: &ChessBoard, border: fn(u64) -> bool, operation: fn(u64) -> u64) {
        let mut copy = position;
        'outer: loop {
            if border(copy) {
                break;
            }

            copy = operation(copy);
            //Check global board
            if board.board[12] & copy != 0 {
                //Check enemy pieces
                for i in (!team).pieces() {
                    if board.board[i] & copy != 0 {
                        output.push(copy);
                        continue 'outer;
                    }
                }
                break
            } else {
                output.push(copy);
            }
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
        }
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