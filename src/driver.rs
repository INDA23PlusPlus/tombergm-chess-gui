pub trait GameDriver
{
	fn features(self: & Self) -> Vec<cnp::Features>;
	fn board(self: & Self) -> [[cnp::Piece; 8]; 8];
	fn joever(self: & Self) -> cnp::Joever;
	fn color(self: & Self) -> cnp::Color;
	fn turn(self: & Self) -> cnp::Color;
	fn moves(self: & Self) -> Vec<cnp::Move>;
	fn get_next_move(self: & Self) -> Option<cnp::Move>;
	fn set_next_move(self: & Self, m: Option<cnp::Move>);
	fn message(self: & Self) -> String;
	fn quit(self: & mut Self);
}
