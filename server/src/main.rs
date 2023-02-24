use chess_engine::{ChessBoard, GameStatus, Pieces, Teams};
use chess_engine::notation::parse_notation;

pub fn main() {
    let board = ChessBoard::new();
    //11111111
    //11111111
    //00000000
    //00000000
    //00000000
    //00030003
    //11111131
    //11111211

    //10000_00000000_00000000, 10_00000000, 1_00000000_00000000
    println!("{}", Pieces::Bishop.get_moves(0b100, Teams::White, &board, false).iter()
        .map(|found_move| format!("{:b}, ", found_move)).collect::<String>());
}

fn notation_test() {
    let board = parse_notation("1. e4 c5 2. d3 f6 3. f3 d6 4. Be2 d5 5. exd5 e6 6. dxe6 Qd6 7. d4 Qxe6 8. f4 Qd6 9. Bb5+ Bd7 10. Bxd7+ Qxd7 11. Qe2+ Kd8 12. Nf3 g5 13. Ne5 Be7 14. Qc4 a6 15. Qxc5 g4 16. Rf1 g3 17. Rf3 h6 18. Rxg3 h5 19. Rxg8+ Bf8 20. Rg3 h4 21. Re3 f5 22. Nf7+ Qxf7 23. Nc3 Rh7 24. Nd5 Rh8 25. Nf6 Rh6 26. Nh7 Rh5 27. Ng5 Rh6 28. d5 Rh8 29. Re7 Rh7 30. Nxf7+ Rxf7 31. Rxf7 Ke8 32. Qe7+ Bxe7 33. d6 Bf6 34. d7+ Kxf7 35. d8=Q Nc6 36. c4 Nb4 37. c5 Nxa2 38. c6 Nb4 39. c7 Nc2+ 40. Kd2 a5 41. c8=Q Kg7 42. Qh8+ Kf7 43. Qxa8 b6 44. Qxa5 b5 45. Qxb5 Be5 46. Qd5+ Ke7 47. Qdxe5+ Kf7 48. Qxf5+ Ke7 49. Qxc2 h3 50. gxh3 Ke6 51. Qd3 Ke7 52. Qg8 Kf6 53. Qe4");

    if let GameStatus::DrawByStalemate = board.game_status {
        println!("Success!");
    } else {
        println!("Failure!");
    }
}
