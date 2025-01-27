use super::values;
use crate::flex;

use super::client::Client;
use super::node::{Node, NodeError};

pub struct Spatial<'a> {
	node: Node<'a>,
}

impl<'a> Spatial<'a> {
	pub fn create(
		client: &Client<'a>,
		spatial_parent: &Spatial<'a>,
		position: values::Vec3,
		rotation: values::Quat,
		scale: values::Vec3,
		translatable: bool,
		rotatable: bool,
		scalable: bool,
		zoneable: bool,
	) -> Result<Self, NodeError> {
		let (node, id) = Node::generate_with_parent(client, "/spatial/spatial")?;

		node.messenger
			.upgrade()
			.ok_or(NodeError::InvalidMessenger)?
			.send_remote_signal(
				"/spatial",
				"createSpatial",
				flex::flexbuffer_from_arguments(|fbb| {
					let mut vec = fbb.start_vector();
					vec.push(id.as_str());
					vec.push(spatial_parent.node.get_path().to_owned().as_str());
					flex_from_vec3!(vec, position);
					flex_from_quat!(vec, rotation);
					flex_from_vec3!(vec, scale);
					vec.push(translatable);
					vec.push(rotatable);
					vec.push(scalable);
					vec.push(zoneable);
					vec.end_vector();
				})
				.as_slice(),
			)
			.map_err(|_| NodeError::ServerCreationFailed)?;

		Ok(Spatial { node })
	}

	pub fn from_path(client: &Client<'a>, path: &str) -> Result<Self, NodeError> {
		Ok(Spatial {
			node: Node::from_path(client, path)?,
		})
	}

	pub fn get_transform(
		&self,
		space: &Spatial,
		callback: impl Fn(values::Vec3, values::Quat, values::Vec3) + 'a,
	) -> Result<(), NodeError> {
		self.node.execute_remote_method(
			"getTransform",
			flex::flexbuffer_from_arguments(|fbb| fbb.build_singleton(space.node.get_path()))
				.as_slice(),
			Box::new(move |data| {
				let root = flexbuffers::Reader::get_root(data).unwrap();
				let flex_vec = root.get_vector().unwrap();
				let pos = flex_to_vec3!(flex_vec.idx(0));
				let rot = flex_to_quat!(flex_vec.idx(1));
				let scl = flex_to_vec3!(flex_vec.idx(2));
				callback(pos, rot, scl);
			}),
		)
	}
	pub fn set_transform(
		&self,
		space: &Spatial,
		position: Option<values::Vec3>,
		rotation: Option<values::Quat>,
		scale: Option<values::Vec3>,
	) -> Result<(), NodeError> {
		self.node.send_remote_signal(
			"setTransform",
			flex::flexbuffer_from_arguments(|fbb| {
				let mut vec = fbb.start_vector();
				vec.push(space.node.get_path());
				if position.as_ref().is_some() {
					flex_from_vec3!(vec, position.as_ref().unwrap());
				}
				if rotation.as_ref().is_some() {
					flex_from_quat!(vec, rotation.as_ref().unwrap());
				}
				if scale.as_ref().is_some() {
					flex_from_vec3!(vec, scale.as_ref().unwrap());
				}
				vec.end_vector();
			})
			.as_slice(),
		)
	}
}
