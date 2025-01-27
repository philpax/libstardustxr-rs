// macro_rules! impl_Coordinates {
// 	($T:ident) => {
// 		impl Coordinates for $T {
// 			fn coordinate(&self) -> (f64, f64) { (self.x, self.y) }
// 		}
// 	}
// }

use super::client::Client;
use crate::flex;
use crate::messenger::Messenger;

use nanoid::nanoid;
use std::{collections::HashMap, rc::Weak, vec::Vec};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NodeError {
	#[error("server creation failed")]
	ServerCreationFailed,
	#[error("messenger is invalid")]
	InvalidMessenger,
	#[error("messenger write failed")]
	MessengerWrite,
	#[error("invalid path")]
	InvalidPath,
	#[error("node doesn't exist")]
	NodeNotFound,
	#[error("method doesn't exist")]
	MethodNotFound,
}

pub struct Node<'a> {
	path: String,
	trailing_slash_pos: usize,
	pub messenger: Weak<Messenger<'a>>,
	local_signals: HashMap<String, Box<dyn Fn(&[u8]) + 'a>>,
	local_methods: HashMap<String, Box<dyn Fn(&[u8]) -> Vec<u8> + 'a>>,
}

impl<'a> Node<'a> {
	pub fn get_name(&self) -> &str {
		&self.path[self.trailing_slash_pos + 1..]
	}
	pub fn get_path(&self) -> &str {
		self.path.as_str()
	}

	pub fn from_path(client: &Client<'a>, path: &str) -> Result<Self, NodeError> {
		if !path.starts_with('/') {
			return Err(NodeError::InvalidPath);
		}
		let node = Node {
			path: path.to_string(),
			trailing_slash_pos: path.rfind('/').ok_or(NodeError::InvalidPath)?,
			messenger: client.get_weak_messenger(),
			local_signals: HashMap::new(),
			local_methods: HashMap::new(),
		};

		// client.scenegraph.
		Ok(node)
	}
	pub fn generate_with_parent(
		client: &Client<'a>,
		parent: &str,
	) -> Result<(Self, String), NodeError> {
		let id = nanoid!(10);
		let mut path = parent.to_string();
		let trailing_slash_pos = path.len();
		if !path.starts_with('/') {
			return Err(NodeError::InvalidPath);
		}
		if !path.ends_with('/') {
			path.push('/');
		}
		path.push_str(&id);

		Ok((
			Node {
				path,
				trailing_slash_pos,
				messenger: client.get_weak_messenger(),
				local_signals: HashMap::new(),
				local_methods: HashMap::new(),
			},
			id,
		))
	}

	pub fn send_local_signal(&self, method: &str, data: &[u8]) -> Result<(), NodeError> {
		self.local_signals
			.get(method)
			.ok_or(NodeError::MethodNotFound)?(data);
		Ok(())
	}
	pub fn execute_local_method(&self, method: &str, data: &[u8]) -> Result<Vec<u8>, NodeError> {
		Ok(self
			.local_methods
			.get(method)
			.ok_or(NodeError::MethodNotFound)?(data))
	}
	pub fn send_remote_signal(&self, method: &str, data: &[u8]) -> Result<(), NodeError> {
		Ok(self
			.messenger
			.upgrade()
			.ok_or(NodeError::InvalidMessenger)?
			.send_remote_signal(self.path.as_str(), method, data)
			.map_err(|_| NodeError::MessengerWrite)?)
	}
	pub fn execute_remote_method(
		&self,
		method: &str,
		data: &[u8],
		callback: Box<dyn Fn(&[u8]) + 'a>,
	) -> Result<(), NodeError> {
		Ok(self
			.messenger
			.upgrade()
			.ok_or(NodeError::InvalidMessenger)?
			.execute_remote_method(self.path.as_str(), method, data, callback)
			.map_err(|_| NodeError::MessengerWrite)?)
	}

	fn destroy(&self) -> Result<(), NodeError> {
		self.send_remote_signal("destroy", &[0; 0])
	}
	fn set_enabled(&self, enabled: bool) -> Result<(), NodeError> {
		self.send_remote_signal(
			"setEnabled",
			flex::flexbuffer_from_arguments(|fbb| fbb.build_singleton(enabled)).as_slice(),
		)
	}
}
