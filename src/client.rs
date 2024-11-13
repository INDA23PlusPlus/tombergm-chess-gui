extern crate scrappy_chess;
extern crate serde;

use crate::driver::GameDriver;
use std::net::TcpStream;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use crate::util::*;

struct State
{
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
		Self
		{
			features	: Vec::new(),
			board		: default_board(),
			joever		: cnp::Joever::Ongoing,
			color		: cnp::Color::White,
			turn		: cnp::Color::White,
			moves		: Vec::new(),
			next_move	: None,

			stream		: None,
			message		: String::new(),
			quit		: false,
		}
	}

	fn pass_turn(self: & mut Self)
	{
		self.turn = inv_color(& self.turn);
	}
}

struct Client
{
	stream		: TcpStream,
	state_rc	: Arc<Mutex<State>>,
}

impl Client
{
	/* Receive and deserialize a data type from the server */
	fn read<T>(self: & Self)
		-> Result<T, serde_json::Error>
		where T: for<'de> serde::de::Deserialize<'de>
	{
		let mut de = serde_json::Deserializer
			::from_reader(& self.stream);

		T::deserialize(& mut de)
	}

	/* Serialize and send a data type to the server */
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

pub struct ClientDriver
{
	thread		: Option<JoinHandle<Result<(), serde_json::Error>>>,
	state_rc	: Arc<Mutex<State>>,
}

impl GameDriver for ClientDriver
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

fn parse_msg(m: & cnp::ServerToClient, s: & mut State)
{
	/* Copy data received in a server message to the local state */
	match m
	{
		cnp::ServerToClient::State
		{
			board,
			moves,
			joever,
			..
		} =>
		{
			s.board = *board;
			s.moves = moves.clone();
			s.joever = *joever;
		},
		cnp::ServerToClient::Error
		{
			board,
			moves,
			joever,
			..
		} =>
		{
			s.board = *board;
			s.moves = moves.clone();
			s.joever = *joever;
		},
		cnp::ServerToClient::Resigned
		{
			board,
			joever,
			..
		} =>
		{
			s.board = *board;
			s.joever = *joever;
		},
		cnp::ServerToClient::Draw
		{
			board,
			moves,
			..
		} =>
		{
			s.board = *board;
			s.moves = moves.clone();
		},
	}
}

fn recv_msg(client: & Client)
	-> Result<cnp::ServerToClient, serde_json::Error>
{
	/* Receive a message from the server */
	let msg = client.read::<cnp::ServerToClient>()?;
	println!("Message received");

	/* Update the state with the received message */
	client.mutate(|state| parse_msg(& msg, state));

	Ok(msg)
}

fn send_move(client: & Client)
	-> Result<(), serde_json::Error>
{
	/* Check if there is a move to send */
	let next_move = client.mutate(|state| state.next_move);

	if let Some(m) = next_move
	{
		/* Send the next move */
		client.write(& cnp::ClientToServer::Move(m))?;
		println!("Move sent");

		/* Receive the server's respone */
		println!("Waiting for response");
		match recv_msg(client)?
		{
			cnp::ServerToClient::State {..} =>
			{
				client.mutate(|state| state.pass_turn());
			},
			_ => (),
		}

		/* Clear the next move */
		client.mutate(|state| state.next_move = None);
	}

	Ok(())
}

fn recv_move(client: & Client)
	-> Result<(), serde_json::Error>
{
	/* Receive the server's next move */
	println!("Waiting for move");
	recv_msg(client)?;

	client.mutate(|state| state.pass_turn());

	Ok(())
}

fn client_main(mut addr: String, state_rc: Arc<Mutex<State>>)
	-> Result<(), serde_json::Error>
{
	println!("Entered client_main");

	/* Add port number to the given address */
	if !addr.contains(':')
	{
		addr.push_str(":8384");
	}

	{
		let mut lock = state_rc.lock().unwrap();
		let state = lock.deref_mut();

		state.message = String::from("Connecting");
	}

	/* Connect */
	let stream_result = std::net::TcpStream::connect(addr);
	if stream_result.is_err()
	{
		return Ok(())
	}
	let stream = stream_result.unwrap();

	/* Create the client struct */
	let client = Client
	{
		stream,
		state_rc,
	};

	client.mutate
	(
		|state| state.message
		= String::from("Connected, sending handshakde")
	);

	/* Clone the stream to the state so that the owner of the driver can
	 * shut it down if they want to quit */
	(
		|stream|
		{
			client.mutate
			(
				|state| state.stream = Some(stream)
			)
		}
	)(client.stream.try_clone().unwrap());

	/* Set the player colors */
	let (_color, opponent) = client.mutate
	(
		|state|
		{
			state.color = cnp::Color::White;

			(
				copy_color(& state.color),
				inv_color(& state.color),
			)
		}
	);

	/* Send client handshake */
	let h = cnp::ClientToServerHandshake
	{
		server_color	: opponent,
	};
	client.write(& h)?;
	println!("Client handshake sent");

	/* Receive server handshake */
	let h = client.read::<cnp::ServerToClientHandshake>()?;
	println!("Server handshake received");

	/* Update state with handshake data */
	client.mutate
	(
		|state|
		{
			state.features = h.features;
			state.board = h.board;
			state.moves = h.moves;
			state.joever = h.joever;
		}
	);

	client.mutate(|state| state.message = String::new());

	/* Move loop */
	while !client.mutate(|state| state.quit)
	{
		if client.mutate(|state| state.turn == state.color)
		{
			send_move(& client)?;
		}
		else
		{
			recv_move(& client)?;
		}
	}

	Ok(())
}

impl ClientDriver
{
	pub fn new(addr: String) -> Self
	{
		/* Create an empty state wrapped in an Arc Mutex */
		let state_rc = Arc::new(Mutex::new(State::new()));

		/* Spin up a driver thread and pass the argument and state to
		 * it */
		let thread =
		(
			|state_rc|
			{
				std::thread::spawn
				(
					|| client_main(addr, state_rc)
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
