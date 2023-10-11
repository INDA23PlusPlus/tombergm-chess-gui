extern crate scrappy_chess;

use self::scrappy_chess::chess;
use self::scrappy_chess::util as chess_util;

pub fn copy_color(c: & cnp::Color) -> cnp::Color
{
	match c
	{
		cnp::Color::White => cnp::Color::White,
		cnp::Color::Black => cnp::Color::Black,
	}
}

pub fn inv_color(c: & cnp::Color) -> cnp::Color
{
	match c
	{
		cnp::Color::White => cnp::Color::Black,
		cnp::Color::Black => cnp::Color::White,
	}
}

pub fn char_to_piece(c: u8) -> cnp::Piece
{
	match c as char
	{
		'K' => cnp::Piece::WhiteKing,
		'Q' => cnp::Piece::WhiteQueen,
		'R' => cnp::Piece::WhiteRook,
		'B' => cnp::Piece::WhiteBishop,
		'N' => cnp::Piece::WhiteKnight,
		'P' => cnp::Piece::WhitePawn,
		'k' => cnp::Piece::BlackKing,
		'q' => cnp::Piece::BlackQueen,
		'r' => cnp::Piece::BlackRook,
		'b' => cnp::Piece::BlackBishop,
		'n' => cnp::Piece::BlackKnight,
		'p' => cnp::Piece::BlackPawn,
		_   => cnp::Piece::None,
	}
}

pub fn translate_move(m: & chess::Move) -> cnp::Move
{
	let start_x = 7 - ((m.from >> 0) & 0x7);
	let start_y = 7 - ((m.from >> 3) & 0x7);
	let end_x = 7 - ((m.to >> 0) & 0x7);
	let end_y = 7 - ((m.to >> 3) & 0x7);

	cnp::Move
	{
		start_x		: start_x as usize,
		start_y		: start_y as usize,
		end_x		: end_x as usize,
		end_y		: end_y as usize,
		promotion	: cnp::Piece::None,
	}
}

pub fn match_move(m: & cnp::Move, c_moves: & Vec<chess::Move>)
	-> Option<chess::Move>
{
	for cm in c_moves
	{
		if *m == translate_move(cm)
		{
			let cm = chess::Move
			{
				result	: cm.result,
				from	: cm.from,
				to	: cm.to,
			};
			return Some(cm);
		}
	}

	None
}

pub fn translate_board(c_state: & chess::ChessState) -> [[cnp::Piece; 8]; 8]
{
	let c_board = chess_util::state_to_ascii(c_state);
	let mut board = [[cnp::Piece::None; 8]; 8];

	for i in 0..64usize
	{
		board[i / 8][i % 8] = char_to_piece(c_board[i]);
	}

	board
}

pub fn default_board() -> [[cnp::Piece; 8]; 8]
{
	translate_board(& chess::ChessState::default())
}

#[derive(Copy, Clone, PartialEq)]
#[allow(dead_code)]
pub enum PieceKind
{
	King,
	Queen,
	Rook,
	Bishop,
	Knight,
	Pawn,
}

pub fn piece_is_kind(p: & cnp::Piece, k: PieceKind) -> bool
{
	match p
	{
		cnp::Piece::WhiteKing | cnp::Piece::BlackKing =>
		{
			match k
			{
				PieceKind::King => true,
				_ => false,
			}
		},
		cnp::Piece::WhiteQueen | cnp::Piece::BlackQueen =>
		{
			match k
			{
				PieceKind::Queen => true,
				_ => false,
			}
		},
		cnp::Piece::WhiteRook | cnp::Piece::BlackRook =>
		{
			match k
			{
				PieceKind::Rook => true,
				_ => false,
			}
		},
		cnp::Piece::WhiteBishop | cnp::Piece::BlackBishop =>
		{
			match k
			{
				PieceKind::Bishop => true,
				_ => false,
			}
		},
		cnp::Piece::WhiteKnight | cnp::Piece::BlackKnight =>
		{
			match k
			{
				PieceKind::Knight => true,
				_ => false,
			}
		},
		cnp::Piece::WhitePawn | cnp::Piece::BlackPawn =>
		{
			match k
			{
				PieceKind::Pawn => true,
				_ => false,
			}
		},
		cnp::Piece::None => false,
	}
}

pub fn piece_from_kind(c: & cnp::Color, k: PieceKind) -> cnp::Piece
{
	match k
	{
		PieceKind::King =>
		{
			match c
			{
				cnp::Color::White => cnp::Piece::WhiteKing,
				cnp::Color::Black => cnp::Piece::BlackKing,
			}
		},
		PieceKind::Queen =>
		{
			match c
			{
				cnp::Color::White => cnp::Piece::WhiteQueen,
				cnp::Color::Black => cnp::Piece::BlackQueen,
			}
		},
		PieceKind::Rook =>
		{
			match c
			{
				cnp::Color::White => cnp::Piece::WhiteRook,
				cnp::Color::Black => cnp::Piece::BlackRook,
			}
		},
		PieceKind::Bishop =>
		{
			match c
			{
				cnp::Color::White => cnp::Piece::WhiteBishop,
				cnp::Color::Black => cnp::Piece::BlackBishop,
			}
		},
		PieceKind::Knight =>
		{
			match c
			{
				cnp::Color::White => cnp::Piece::WhiteKnight,
				cnp::Color::Black => cnp::Piece::BlackKnight,
			}
		},
		PieceKind::Pawn =>
		{
			match c
			{
				cnp::Color::White => cnp::Piece::WhitePawn,
				cnp::Color::Black => cnp::Piece::BlackPawn,
			}
		},
	}
}
