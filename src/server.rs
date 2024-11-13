extern crate scrappy_chess;
extern crate serde;

use crate::driver::GameDriver;
use self::scrappy_chess::chess;
use std::net::TcpStream;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use crate::util::*;

struct State
{
	c_state		: chess::ChessState,
	c_moveset	: chess::MoveSet,
	c_moves		: Vec<chess::Move>,

	features	: Vec<cnp::Features>,
	board		: [[cnp::Piece; 8]; 8],
	joever		: cnp::Joever,
	color		: cnp::Color,
	turn		: cnp::Color,
	moves		: Vec<cnp::Move>,
	next_move	: Option<cnp::Move>,

	stream		: Option<TcpStream>,
	message		: String,
	quit		: bool,
}

impl State
{
	fn new() -> Self
	{
		let mut state = Self
		{
			c_state		: chess::ChessState::standard(),
			c_moveset	: chess::MoveSet::new(),
			c_moves		: Vec::new(),

			features	: vec!
				[
					cnp::Features::PossibleMoveGeneration,
				],
			board		: [[cnp::Piece::None; 8]; 8],
			joever		: cnp::Joever::Ongoing,
			color		: cnp::Color::White,
			turn		: cnp::Color::White,
			moves		: Vec::new(),
			next_move	: None,

			stream		: None,
			message		: String::new(),
			quit		: false,
		};

		state.update();

		state
	}

	fn pass_turn(self: & mut Self)
	{
		self.turn = inv_color(& self.turn);
	}

	fn update(self: & mut Self)
	{
		self.board = translate_board(& self.c_state);

		self.c_moves = self.c_state.get_moves(& self.c_moveset);
		self.moves = self.c_moves.iter().map(translate_move).collect();

		if self.moves.len() == 0
		{
			self.joever = cnp::Joever::Indeterminate;
		}
	}
}

struct Server
{
	stream		: TcpStream,
	state_rc	: Arc<Mutex<State>>,
}

impl Server
{
	/* Receive and deserialize a data type from the client */
	fn read<T>(self: & Self)
		-> Result<T, serde_json::Error>
		where T: for<'de> serde::de::Deserialize<'de>
	{
		let mut de = serde_json::Deserializer
			::from_reader(& self.stream);

		T::deserialize(& mut de)
	}

	/* Serialize and send a data type to the client */
	fn write<T>(self: & Self, t: & T)
		-> Result<(), serde_json::Error>
		where T: serde::ser::Serialize
	{
		serde_json::to_writer(& self.stream, t)
	}

	/* Acquire and mutate the local state in a closure */
	fn mutate<F, R>(self: & Self, f: F)
		-> R
		where F: FnOnce(& mut State) -> R
	{
		let mut lock = self.state_rc.lock().unwrap();
		
		f(lock.deref_mut())
	}
}

pub struct ServerDriver
{
	thread		: Option<JoinHandle<Result<(), serde_json::Error>>>,
	state_rc	: Arc<Mutex<State>>,
}

impl GameDriver for ServerDriver
{
	fn features(self: & Self) -> Vec<cnp::Features>
	{
		let mut lock = self.state_rc.lock().unwrap();
		let state = lock.deref_mut();

		state.features.clone()
	}

	fn board(self: & Self) -> [[cnp::Piece; 8]; 8]
	{
		let mut lock = self.state_rc.lock().unwrap();
		let state = lock.deref_mut();

		state.board
	}

	fn joever(self: & Self) -> cnp::Joever
	{
		let mut lock = self.state_rc.lock().unwrap();
		let state = lock.deref_mut();
	
		state.joever
	}

	fn color(self: & Self) -> cnp::Color
	{
		let mut lock = self.state_rc.lock().unwrap();
		let state = lock.deref_mut();
	
		copy_color(& state.color)
	}

	fn turn(self: & Self) -> cnp::Color
	{
		let mut lock = self.state_rc.lock().unwrap();
		let state = lock.deref_mut();
	
		copy_color(& state.turn)
	}

	fn moves(self: & Self) -> Vec<cnp::Move>
	{
		let mut lock = self.state_rc.lock().unwrap();
		let state = lock.deref_mut();
	
		state.moves.clone()
	}

	fn get_next_move(self: & Self) -> Option<cnp::Move>
	{
		let mut lock = self.state_rc.lock().unwrap();
		let state = lock.deref_mut();

		state.next_move
	}

	fn set_next_move(self: & Self, m: Option<cnp::Move>)
	{
		let mut lock = self.state_rc.lock().unwrap();
		let state = lock.deref_mut();

		state.next_move = m;
	}

	fn message(self: & Self) -> String
	{
		let mut lock = self.state_rc.lock().unwrap();
		let state = lock.deref_mut();

		state.message.clone()
	}

	fn quit(self: & mut Self)
	{
		{
			let mut lock = self.state_rc.lock().unwrap();
			let state = lock.deref_mut();
	
			if let Some(ref stream) = state.stream
			{
				let how = std::net::Shutdown::Both;
				let _ = stream.shutdown(how);

				state.stream = None;
			}

			state.quit = true;
		}

		if self.thread.is_some()
		{
			let _ = self.thread.take().unwrap().join();
		}
	}
}

fn send_state(server: & Server, move_made: cnp::Move)
	-> Result<(), serde_json::Error>
{
	let msg = server.mutate
	(
		|state|
		cnp::ServerToClient::State
		{
			board		: state.board,
			moves		: state.moves.clone(),
			joever		: state.joever,
			move_made
		}
	);

	server.write(& msg)?;
	println!("State sent");

	Ok(())
}

fn send_err(server: & Server, message: & str)
	-> Result<(), serde_json::Error>
{
	let msg = server.mutate
	(
		|state|
		cnp::ServerToClient::Error
		{
			board		: state.board,
			moves		: state.moves.clone(),
			joever		: state.joever,
			message		: String::from(message),
		}
	);

	server.write(& msg)?;
	println!("Error sent: {}", message);

	Ok(())
}

fn send_move(server: & Server)
	-> Result<(), serde_json::Error>
{
	/* Validate next move */
	let next_move = server.mutate
	(
		|state|
		{
			if let Some(m) = state.next_move.take()
			{
				let ms = & state.c_moves;

				if let Some(cm) = match_move(& m, ms)
				{
					return Some((m, cm));
				}
			}

			None
		}
	);

	if let Some((m, cm)) = next_move
	{
		/* Play move */
		server.mutate
		(
			|state|
			{
				state.c_state = cm.result;
				state.update();
				state.pass_turn();
			}
		);

		/* Send updated state to client */
		send_state(server, m)?;
	}

	Ok(())
}

fn recv_move(server: & Server)
	-> Result<(), serde_json::Error>
{
	/* Receive message from client */
	println!("Waiting for move");
	let msg = server.read::<cnp::ClientToServer>()?;
	println!("Message received");

	if let cnp::ClientToServer::Move(m) = msg
	{
		println!("Move received");

		/* Validate received move */
		let next_move = server.mutate
		(
			|state|
			{
				let ms = & state.c_moves;

				if let Some(cm) = match_move(& m, ms)
				{
					return Some((m, cm));
				}

				None
			}
		);

		if let Some((m, cm)) = next_move		
		{
			/* Play move */
			server.mutate
			(
				|state|
				{
					state.c_state = cm.result;
					state.update();
					state.pass_turn();
				}
			);

			/* Send updated state to the client */
			send_state(server, m)?;
		}
		else
		{
			/* Send error message */
			send_err(server, "That move is invalid")?;
		}
	}
	else
	{
		/* Unsupported, send error message */
		send_err(server, "That action is not supported")?;
	}

	Ok(())
}

fn server_main(state_rc: Arc<Mutex<State>>)
	-> Result<(), serde_json::Error>
{
	println!("Entered server_main");

	{
		let mut lock = state_rc.lock().unwrap();
		let state = lock.deref_mut();

		state.message = String::from("Starting network game");
	}

	/* Listen for a connection */
	let bind_result = std::net::TcpListener::bind("0.0.0.0:8384");
	if bind_result.is_err()
	{
		return Ok(());
	}
	let listener = bind_result.unwrap();
	println!("Server listening");

	{
		let mut lock = state_rc.lock().unwrap();
		let state = lock.deref_mut();

		state.message = String::from("Waiting for opponent");
	}
	
	/* Accept a connection */
	let accept_result = listener.accept();
	if accept_result.is_err()
	{
		return Ok(());
	}
	let (stream, _addr) = accept_result.unwrap();
	println!("Server connected");

	/* Create the server struct */
	let server = Server
	{
		stream,
		state_rc,
	};

	server.mutate
	(
		|state| state.message
		= String::from("Opponent connected, waiting for handshake")
	);

	/* Clone the stream to the state so that the owner of the driver can
	 * shut it down if they want to quit */
	(
		|stream|
		{
			server.mutate
			(
				|state| state.stream = Some(stream)
			)
		}
	)(server.stream.try_clone().unwrap());

	/* Receive client handshake */
	let h = server.read::<cnp::ClientToServerHandshake>()?;
	println!("Client handshake received");

	/* Set player color */
	server.mutate
	(
		|state|
		{
			state.color = copy_color(& h.server_color);
		}
	);

	/* Send server handshake */
	let h = server.mutate
	(
		|state|
		{
			cnp::ServerToClientHandshake
			{
				features	: state.features.clone(),
				board		: state.board,
				moves		: state.moves.clone(),
				joever		: state.joever,
			}
		}
	);
	server.write(& h)?;
	println!("Server handshake sent");

	server.mutate(|state| state.message = String::new());

	/* Move loop */
	while !server.mutate(|state| state.quit)
	{
		if server.mutate(|state| state.turn == state.color)
		{
			send_move(& server)?;
		}
		else
		{
			recv_move(& server)?;
		}
	}

	Ok(())
}

impl ServerDriver
{
	pub fn new() -> Self
	{
		/* Create an empty state wrapped in an Arc Mutex */
		let state_rc = Arc::new(Mutex::new(State::new()));
		
		/* Spin up a driver thread and pass the state to it */
		let thread =
		(
			|state_rc|
			{
				std::thread::spawn
				(
					|| server_main(state_rc)
				)
			}
		)(state_rc.clone());

		/* Create the driver handle and return it */
		Self
		{
			thread		: Some(thread),
			state_rc,
		}
	}
}
