use crate::{ChessBoard, Pieces, Teams};
use crate::util::Position;

pub fn parse_notation(moves: &str) -> ChessBoard {
    let mut board = ChessBoard::new();

    let mut temp = moves.to_string();
    while !temp.is_empty() {
        let team = Teams::from(board.move_number);
        println!("{} to move:\n{}", team, board);
        //Remove move number
        if let Teams::White = team {
            temp = temp[3..].to_string();
        }
        let mut found_move = temp[..*temp.find(' ').get_or_insert(temp.len())].to_string();
        if found_move.contains('+') {
            found_move = found_move[..found_move.len()-1].to_string();
        }
        let piece = Pieces::from(found_move.chars().nth(0).unwrap());
        let target;
        if found_move.contains('x') {
            target = Position::from(&found_move[found_move.find('x').unwrap()+1..found_move.find('x').unwrap()+3]);
        } else {
            match piece {
                Pieces::Pawn => target = Position::from(&found_move[0..2]),
                _ => target = Position::from(&found_move[1..3])
            }
        }
        let mut promoting = found_move.find('=').map(|start| Pieces::from(found_move.chars().nth(start+1).unwrap()));

        println!("{}: {} = {}", found_move, target, promoting.get_or_insert(Pieces::Pawn));
        temp = temp[temp.find(' ').unwrap()+1..].to_string();

        let mut moved = false;
        println!("Target: {}", target);
        for position in board.get_pieces(piece as usize + team as usize) {
            println!("Moving {}: {}", position, piece.get_moves(position, team, &board, false)
                .iter().map(|found| format!("{}, ", found)).collect::<String>());

            if piece.get_moves(position, team, &board, false)
                .contains(&target) {
                if !board.move_piece(piece, team, position, target, promoting) {
                    panic!("Failed a found move!");
                }
                moved = true;
            }
        }

        if !moved {
            panic!("Failed to find a move!");
        }
    }
    return board;
}