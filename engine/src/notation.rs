use crate::{ChessBoard, convert_to_flat, Pieces, Teams};

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
        let piece = to_piece(found_move.chars().nth(0).unwrap());
        let target;
        if found_move.contains('x') {
            target = to_location(&found_move[found_move.find('x').unwrap()+1..found_move.find('x').unwrap()+3]);
        } else {
            match piece {
                Pieces::Pawn => target = to_location(&found_move[0..2]),
                _ => target = to_location(&found_move[1..3])
            }
        }
        let mut promoting = found_move.find('=').map(|start| to_piece(found_move.chars().nth(start+1).unwrap()));

        println!("{}: {:?} = {}", found_move, target, promoting.get_or_insert(Pieces::Pawn));
        temp = temp[temp.find(' ').unwrap()+1..].to_string();

        let target= (target.0, 7-target.1);
        let mut moved = false;
        println!("Target: {} ({:b})", convert_to_flat(target), convert_to_flat(target));
        for bit_position in board.get_pieces(piece as usize + team as usize) {
            println!("Moving {:b}: {}", bit_position, piece.get_moves(bit_position, team, &board, false)
                .iter().map(|found| format!("{:b}, ", *found)).collect::<String>());

            if piece.get_moves(bit_position, team, &board, false)
                .contains(&convert_to_flat(target)) {
                if !board.move_piece_bits(piece, team, bit_position,
                                          convert_to_flat(target), promoting) {
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

fn to_location(position: &str) -> (u8, u8) {
    return (position.as_bytes()[0] - b'a', position.as_bytes()[1] - b'1')
}

fn to_piece(character: char) -> Pieces {
    return match character {
        'Q' => Pieces::Queen,
        'N' => Pieces::Knight,
        'B' => Pieces::Bishop,
        'R' => Pieces::Rook,
        'K' => Pieces::Rook,
        _ => Pieces::Pawn
    }
}