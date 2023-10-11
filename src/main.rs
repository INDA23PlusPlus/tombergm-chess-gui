extern crate chess_network_protocol as cnp;
extern crate ggez;

mod client;
mod driver;
mod server;
mod util;

use client::ClientDriver;
use driver::GameDriver;
use server::ServerDriver;
use util::{PieceKind, piece_is_kind, piece_from_kind};

fn main()
{
	let driver: Box<dyn GameDriver>;

	if std::env::args().len() == 1
	{
		driver = Box::new(ServerDriver::new());
	}
	else if std::env::args().len() == 2
	{
		let addr = std::env::args().nth(1).unwrap();
		driver = Box::new(ClientDriver::new(addr));
	}
	else
	{
		return;
	}

	let (mut ctx, event_loop) =
		ggez::ContextBuilder::new("chess-gui", "Tommy Bergman")
		.window_setup(ggez::conf::WindowSetup
			{
				title	: String::from("Chess"),
				samples	: ggez::conf::NumSamples::One,
				vsync	: true,
				icon	: String::from(""),
				srgb	: true,
			})
		.backend(ggez::conf::Backend::Gl)
		.build()
		.expect("");

	let game = Game::new(& mut ctx, driver);

	ggez::event::run(ctx, event_loop, game);
}

struct Game
{
	driver	: Box<dyn GameDriver>,
	select	: Option<(i32, i32)>,
	promo	: PieceKind,
	images	: std::collections::HashMap<usize, ggez::graphics::Image>,
}

impl Game
{
	const BOARD_X	: f32 = 100.;
	const BOARD_Y	: f32 = 100.;
	const SQUARE_W	: f32 = 100.;
	const SQUARE_H	: f32 = Self::SQUARE_W;
	const PROMO_X	: f32 = Self::BOARD_X + Self::SQUARE_W * 8. + 50.;
	const PROMO_Y	: f32 = Self::BOARD_Y;

	pub fn new(ctx: & mut ggez::Context, driver: Box<dyn GameDriver>)
		-> Game
	{
		ctx.gfx.window().set_inner_size(
			ggez::winit::dpi::PhysicalSize::new(550., 500.));

		let mut game = Game
		{
			driver,
			select	: None,
			promo	: PieceKind::Queen,
			images	: std::collections::HashMap::new(),
		};

	/* Cburnett,
	 * CC BY-SA 3.0 <http://creativecommons.org/licenses/by-sa/3.0/>,
	 * via Wikimedia Commons
	 */
		for (c, p) in
		[
			(cnp::Piece::WhiteKing,"/Chess_klt45.svg.png"),
			(cnp::Piece::WhiteQueen,"/Chess_qlt45.svg.png"),
			(cnp::Piece::WhiteRook,"/Chess_rlt45.svg.png"),
			(cnp::Piece::WhiteBishop,"/Chess_blt45.svg.png"),
			(cnp::Piece::WhiteKnight,"/Chess_nlt45.svg.png"),
			(cnp::Piece::WhitePawn,"/Chess_plt45.svg.png"),
			(cnp::Piece::BlackKing,"/Chess_kdt45.svg.png"),
			(cnp::Piece::BlackQueen,"/Chess_qdt45.svg.png"),
			(cnp::Piece::BlackRook,"/Chess_rdt45.svg.png"),
			(cnp::Piece::BlackBishop,"/Chess_bdt45.svg.png"),
			(cnp::Piece::BlackKnight,"/Chess_ndt45.svg.png"),
			(cnp::Piece::BlackPawn,"/Chess_pdt45.svg.png"),
		]
		{
			let im = ggez::graphics::Image
				::from_path(ctx, p).unwrap();
			game.images.insert(c as usize, im);
		}

		game
	}

	fn click_board(self: & mut Self, x: f32, y: f32)
		-> ggez::GameResult
	{
		let mut coords =
		(
			(x / Self::SQUARE_W) as i32,
			(y / Self::SQUARE_H) as i32,
		);
		let mut promo_rank = 7;

		if self.driver.color() == cnp::Color::Black
		{
			coords.1 = 7 - coords.1;
			promo_rank = 0;
		}

		if Some(coords) == self.select
		{
			self.select = None;
		}
		else if let Some(select) = self.select
		{
			self.select = Some(coords);

			let board = self.driver.board();
			let piece = board
				[select.1 as usize]
				[select.0 as usize];
			let mut promo = cnp::Piece::None;

			if coords.1 == promo_rank
				&& piece_is_kind(& piece, PieceKind::Pawn)
			{
				promo = piece_from_kind
				(
					& self.driver.color(),
					self.promo,
				);
			}
	
			let m = Some
			(
				cnp::Move
				{
					start_x		: select.0 as usize,
					start_y		: select.1 as usize,
					end_x		: coords.0 as usize,
					end_y		: coords.1 as usize,
					promotion	: promo,
				}
			);

			if self.driver.turn() == self.driver.color()
				&& self.driver.features().contains
				(& cnp::Features::PossibleMoveGeneration)
			{
				for n in self.driver.moves()
				{
					if m == Some(n)
					{
						self.driver.set_next_move(m);
						self.select = None;

						break;
					}
				}
			}
			else
			{
				self.driver.set_next_move(m);
				self.select = None;
			}
		}
		else
		{
			self.driver.set_next_move(None);
			self.select = Some(coords);
		}

		Ok(())
	}

	fn click_promo(self: & mut Self, _x: f32, y: f32)
		-> ggez::GameResult
	{
		let y = (y / Self::SQUARE_H) as i32;

		let promo_kinds =
		[
			PieceKind::Queen,
			PieceKind::Rook,
			PieceKind::Bishop,
			PieceKind::Knight,
		];

		self.promo = promo_kinds[y as usize];

		Ok(())
	}
}


impl ggez::event::EventHandler for Game
{
	fn mouse_button_down_event(self: & mut Self,
					_ctx: & mut ggez::Context,
					button: ggez::event::MouseButton,
					x: f32, y: f32)
		-> ggez::GameResult
	{
		if self.driver.joever() != cnp::Joever::Ongoing
		{
			Ok(())
		}
		else if button != ggez::event::MouseButton::Left
		{
			Ok(())
		}
		else if x >= Self::BOARD_X
			&& y >= Self::BOARD_Y
			&& x < Self::BOARD_X + Self::SQUARE_W * 8.
			&& y < Self::BOARD_Y + Self::SQUARE_H * 8.
		{
			let x = x - Self::BOARD_X;
			let y = y - Self::BOARD_Y;

			self.click_board(x, y)
		}
		else if x >= Self::PROMO_X
			&& y >= Self::PROMO_Y
			&& x < Self::PROMO_X + Self::SQUARE_W * 1.
			&& y < Self::PROMO_Y + Self::SQUARE_H * 4.
		{
			let x = x - Self::PROMO_X;
			let y = y - Self::PROMO_Y;

			self.click_promo(x, y)
		}
		else
		{
			Ok(())
		}
	}

	fn key_down_event(& mut self,
				ctx: & mut ggez::Context,
				input: ggez::input::keyboard::KeyInput,
				_repeated: bool)
		-> Result<(), ggez::GameError>
	{
		if let Some(kc) = input.keycode
		{
			if kc == ggez::input::keyboard::KeyCode::Escape
			{
				self.driver.quit();
				ctx.request_quit();
			}
		}

		Ok(())
	}

	fn update(self: & mut Self, _ctx: & mut ggez::Context)
		-> ggez::GameResult
	{
		if self.driver.joever() != cnp::Joever::Ongoing
		{
			self.select = None;
		}

		Ok(())
	}

	fn draw(self: & mut Self, ctx: & mut ggez::Context)
		-> ggez::GameResult
	{
		use ggez::graphics::*;

		let mut canvas = Canvas
			::from_frame(ctx, Color::from_rgb(155, 115, 55));

		let square_dp = DrawParam::default()
			.scale([100., 100.]);

		let icon_dp = DrawParam::default()
			.scale([100. / 240., 100. / 240.]);

		let circle = Mesh::new_circle
		(
			ctx,
			DrawMode::Fill(FillOptions::DEFAULT),
			[0., 0.],
			25.,
			0.1,
			Color::from_rgba(128, 128, 128, 128),
		).expect("");
		let circle_dp = DrawParam::default();

		let mut text = Text::new("");
		text.set_scale(50.);
		text.set_layout(TextLayout::center());
		let text_dp = DrawParam::default()
			.color(Color::from_rgb(120, 100, 90));

		let board = self.driver.board();

		for y in 0..8
		{
			for x in 0..8
			{
				let p = board[y as usize][x as usize];

				let dest_ul;
				if self.driver.color() == cnp::Color::White
				{
					dest_ul =
					[
						Self::BOARD_X
							+ Self::SQUARE_W
							* x as f32,
						Self::BOARD_Y
							+ Self::SQUARE_H
							* y as f32,
					];
				}
				else
				{
					dest_ul =
					[
						Self::BOARD_X
							+ Self::SQUARE_W
							* x as f32,
						Self::BOARD_Y
							+ Self::SQUARE_H
							* (7 - y) as f32,
					];
				}
				let dest_c =
				[
					dest_ul[0] + Self::SQUARE_W / 2.,
					dest_ul[1] + Self::SQUARE_H / 2.,
				];

				let mut is_target = false;
				let mut is_select = false;
				let mut is_dest = false;

				if let Some(select) = self.select
				{
					let m = cnp::Move
					{
						start_x		:
							select.0 as usize,
						start_y		:
							select.1 as usize,
						end_x		: x as usize,
						end_y		: y as usize,
						promotion	:
							cnp::Piece::None,
					};

					for mut n in self.driver.moves()
					{
						n.promotion = cnp::Piece::None;

						if m == n
						{
							is_target = true;
							break;
						}
					}

					if x == select.0 && y == select.1
					{
						is_select = true;
					}
				}

				if let Some(m) = self.driver.get_next_move()
				{
					if m.start_x == x as usize
						&& m.start_y == y as usize
					{
						is_select = true;
					}
					else if m.end_x == x as usize
						&& m.end_y == y as usize
					{
						is_dest = true;
					}
				}

				let color;
				if is_dest
				{
					color = Color::from_rgb(210, 150, 150);
				}
				else if is_select
				{
					color = Color::from_rgb(190, 210, 150);
				}
				else if (x + y) % 2 == 0
				{
					color = Color::from_rgb(255, 250, 240);
				}
				else
				{
					color = Color::from_rgb(210, 190, 150);
				}

				canvas.draw
				(
					& Quad,
					square_dp.color(color).dest(dest_ul),
				);

				let im = self.images.get(& (p as usize));
				if let Some(im) = im
				{
					canvas.draw
					(
						im,
						icon_dp.dest(dest_ul),
					);
				}

				if is_target
					&& self.driver.turn()
						== self.driver.color()
				{
					let color;

					if p == cnp::Piece::None
					{
						color = Color::WHITE;
					}
					else
					{
						color = Color::RED;
					}

					canvas.draw
					(
						& circle,
						circle_dp
							.dest(dest_c)
							.color(color),
					);
				}
			}
		}

		{
			let c = self.driver.color();
			let promo_p = piece_from_kind(& c, self.promo);

			for (y, k) in
			[
				(0, PieceKind::Queen),
				(1, PieceKind::Rook),
				(2, PieceKind::Bishop),
				(3, PieceKind::Knight),
			]
			{
				let p = piece_from_kind(& c, k);
				let dest =
				[
					Self::PROMO_X,
					Self::PROMO_Y
						+ Self::SQUARE_H * y as f32,
				];
				
				if p == promo_p
				{
					let color = Color
						::from_rgb(190, 210, 150);
					canvas.draw
					(
						& Quad,
						square_dp
							.color(color)
							.dest(dest),
					);
				}

				let im = self.images.get(& (p as usize));
				if let Some(im) = im
				{
					canvas.draw(im, icon_dp.dest(dest));
				}
			}
		}

		let joever = match self.driver.joever()
		{
			cnp::Joever::Draw
				=> Some("The game is a draw"),
			cnp::Joever::White
				=> Some("White has won!"),
			cnp::Joever::Black
				=> Some("Black has won!"),
			cnp::Joever::Indeterminate
				=> Some("Game over!"),
			cnp::Joever::Ongoing
				=> None,

		};

		if let Some(joever) = joever
		{
			text.clear();
			text.add(joever);
			canvas.draw
			(
				& text,
				text_dp.dest
				([
					Self::BOARD_X
						+ Self::SQUARE_W * 4.,
					Self::BOARD_Y
						+ Self::SQUARE_H * 4.,
				]),
			);
		}

		text.clear();
		text.add(self.driver.message());
		text.set_scale(25.);
		canvas.draw
		(
			& text,
			text_dp
				.dest
				([
					Self::BOARD_X + Self::SQUARE_W * 4.,
					Self::BOARD_Y
						+ Self::SQUARE_H * 8.
						+ 50.,
				])
				.color(Color::BLACK),
		);

		canvas.finish(ctx)
	}

	fn quit_event(self: & mut Self, _ctx: & mut ggez::Context)
		-> ggez::GameResult<bool>
	{
		self.driver.quit();

		Ok(false)
	}
}
