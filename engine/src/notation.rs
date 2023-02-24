use crate::{ChessBoard, convert_back, convert_to_flat, Pieces, Teams};

pub fn parse_notation(moves: &str) -> ChessBoard {
    let mut board = ChessBoard::new();

    let mut temp = moves.to_string();
    while !temp.is_empty() {
        println!("{}", board);
        //Remove move number
        if board.move_number % 2 == 0 {
            temp = temp[3..].to_string();
        }
        let mut found_move = temp[..*temp.find(' ').get_or_insert(temp.len())].to_string();
        println!("{}", found_move);
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
        let promoting = found_move.find('=').map(|start| to_piece(found_move.chars().nth(start+1).unwrap()));

        temp = temp[found_move.len()..].to_string();

        let mut moved = false;
        for position in board.get_pieces(piece as usize) {
            println!("Checking {:?}", convert_back(position));
            if piece.get_moves(position, Teams::from(board.move_number), &board, false).contains(&convert_to_flat(target)) {
                board.move_piece(piece, Teams::from(board.move_number), convert_back(position), target, promoting);
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
    return (position.as_bytes()[0] - b'a', position.as_bytes()[1] - b'0')
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