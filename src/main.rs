extern crate ggez;
extern crate scrappy_chess;

use scrappy_chess::chess;
use scrappy_chess::util as chess_util;

fn main()
{
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

	let game = Game::new(& mut ctx);

	ggez::event::run(ctx, event_loop, game);
}

struct Game
{
	state		: chess::ChessState,
	move_set	: chess::MoveSet,
	moves		: Vec<chess::Move>,
	select		: Option<(i32, i32)>,
	images		: std::collections
				::HashMap<char, ggez::graphics::Image>,
}

impl Game
{
	pub fn new(ctx: & mut ggez::Context) -> Game
	{
		ctx.gfx.window().set_inner_size(
			ggez::winit::dpi::PhysicalSize::new(400., 400.));

		let mut game = Game
		{
			state		: chess::ChessState::standard(),
			move_set	: chess::MoveSet::new(),
			moves		: Vec::new(),
			select		: None,
			images		: std::collections::HashMap::new(),
		};

	/* Cburnett,
	 * CC BY-SA 3.0 <http://creativecommons.org/licenses/by-sa/3.0/>,
	 * via Wikimedia Commons
	 */
		for (c, p) in
			[
				('K', "/Chess_klt45.svg.png"),
				('Q', "/Chess_qlt45.svg.png"),
				('R', "/Chess_rlt45.svg.png"),
				('B', "/Chess_blt45.svg.png"),
				('N', "/Chess_nlt45.svg.png"),
				('P', "/Chess_plt45.svg.png"),
				('k', "/Chess_kdt45.svg.png"),
				('q', "/Chess_qdt45.svg.png"),
				('r', "/Chess_rdt45.svg.png"),
				('b', "/Chess_bdt45.svg.png"),
				('n', "/Chess_ndt45.svg.png"),
				('p', "/Chess_pdt45.svg.png"),
			]
		{
			let im = ggez::graphics::Image
				::from_path(ctx, p).unwrap();
			game.images.insert(c, im);
		}

		game.moves = game.state.get_moves(& game.move_set);

		game
	}
}

fn bit_loc(loc: (i32, i32)) -> u8
{
	(((7 - loc.1) << 3) | (7 - loc.0)) as u8
}

impl ggez::event::EventHandler for Game
{
	fn mouse_button_down_event(self: & mut Self,
					_ctx: & mut ggez::Context,
					button: ggez::event::MouseButton,
					x: f32, y: f32)
		-> ggez::GameResult
	{
		if button != ggez::event::MouseButton::Left
		{
			return Ok(());
		}

		if self.moves.len() == 0
		{
			self.state = chess::ChessState::standard();
			self.moves = self.state.get_moves(& self.move_set);

			return Ok(());
		}

		let coords =
		(
			(x / 100.) as i32,
			(y / 100.) as i32,
		);

		if let Some(select) = self.select
		{
			let from = bit_loc(select);
			let to = bit_loc(coords);

			self.select = Some(coords);

			for m in & self.moves
			{
				if m.from == from && m.to == to
				{
					self.state = m.result;
					self.moves =
					self.state.get_moves(& self.move_set);
					self.select = None;

					break;
				}
			}
		}
		else
		{
			self.select = Some(coords);
		}

		Ok(())
	}

	fn update(self: & mut Self, _ctx: & mut ggez::Context)
		-> ggez::GameResult
	{
		Ok(())
	}

	fn draw(self: & mut Self, ctx: & mut ggez::Context)
		-> ggez::GameResult
	{
		use ggez::graphics::*;

		let mut canvas = Canvas::from_frame(ctx, Color::WHITE);

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

		let board = chess_util::state_to_ascii(& self.state);

		for y in 0..8
		{
			for x in 0..8
			{
				let c = board[(y * 8 + x) as usize] as char;
				let dest_ul =
				[
					100. * x as f32,
					100. * y as f32,
				];
				let dest_c =
				[
					dest_ul[0] + 50.,
					dest_ul[1] + 50.,
				];

				let mut is_target = false;
				let mut is_select = false;

				if let Some(select) = self.select
				{
					let from = bit_loc(select);
					let to = bit_loc((x, y));

					for m in & self.moves
					{
						if m.from == from && m.to == to
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

				let color;
				if is_select
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

				let im = self.images.get(& c);
				if let Some(im) = im
				{
					canvas.draw
					(
						im,
						icon_dp.dest(dest_ul),
					);
				}

				if is_target
				{
					let color;

					if c == ' '
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

				if self.moves.len() == 0
				{
					text.clear();
					text.add("Game over!");
					canvas.draw
					(
						& text,
						text_dp.dest([400., 375.]),
					);

					text.clear();
					text.add("Click anywhere to restart");
					canvas.draw
					(
						& text,
						text_dp.dest([400., 425.]),
					);
				}
			}
		}

		canvas.finish(ctx)
	}
}

