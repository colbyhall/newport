use crate::{
	Engine,
	Event,
	Module,
};

use std::{
	any::{
		Any,
		TypeId,
	},
	collections::HashMap,
};

pub(crate) struct BuilderEntry {
	pub id: TypeId,
	pub spawn: fn() -> Box<dyn Any>,
}

pub trait Register = Sized + Clone + 'static;

/// Structure used to define engine structure and execution
#[derive(Default)]
pub struct Builder {
	pub(crate) entries: Vec<BuilderEntry>,
	pub(crate) name: Option<String>,

	pub(crate) process_input: Vec<Box<dyn Fn(&Event) + 'static>>,
	pub(crate) tick: Vec<Box<dyn Fn(f32) + 'static>>,
	pub(crate) display: Option<Box<dyn Fn() + 'static>>, // There can only be one display method

	pub(crate) registers: Option<HashMap<TypeId, Box<dyn Any>>>,
}

impl Builder {
	/// Creates a new [`Builder`]
	pub fn new() -> Self {
		Self {
			entries: Vec::with_capacity(32),
			name: None,

			process_input: Vec::new(),
			tick: Vec::new(),
			display: None,

			registers: Some(HashMap::new()),
		}
	}

	/// Adds a module to the list
	///
	/// # Arguments
	///
	/// * `T` - A [`Module`] that will be initialized and used at runtime
	///
	/// # Examples
	///
	/// ```
	/// use newport_engine::Builder;
	///
	/// let builder = Builder::new()
	///     .module::<Test>();
	/// ```
	pub fn module<T: Module>(mut self) -> Self {
		// Don't add another module thats already added
		let id = TypeId::of::<T>();
		for it in self.entries.iter() {
			if it.id == id {
				return self;
			}
		}

		fn spawn<T: Module>() -> Box<dyn Any> {
			Box::new(T::new())
		}

		// Add dependencies to the entries list. There will be duplicates
		self = T::depends_on(self);

		// Push entry with generic spawn func and type id
		self.entries.push(BuilderEntry {
			id,
			spawn: spawn::<T>,
		});

		self
	}

	pub fn process_input(mut self, f: impl Fn(&Event) + 'static) -> Self {
		self.process_input.push(Box::new(f));
		self
	}

	/// Adds a tick closure to the list
	pub fn tick(mut self, f: impl Fn(f32) + 'static) -> Self {
		self.tick.push(Box::new(f));
		self
	}

	pub fn display(mut self, f: impl Fn() + 'static) -> Self {
		self.display = Some(Box::new(f));
		self
	}

	/// Sets the name of the engine runnable
	pub fn name(mut self, name: impl Into<String>) -> Self {
		self.name = Some(name.into());
		self
	}

	pub fn register<T: Register>(mut self, register: T) -> Self {
		let type_id = TypeId::of::<T>();
		let registers = self.registers.as_mut().unwrap();
		let it = match registers.get_mut(&type_id) {
			Some(it) => it,
			None => {
				let register: Vec<T> = Vec::new();
				registers.insert(type_id, Box::new(register));
				registers.get_mut(&type_id).unwrap()
			}
		};

		let registers = it.downcast_mut::<Vec<T>>().unwrap();
		registers.push(register);

		self
	}

	// TODO: Document
	pub fn run(self) {
		Engine::run(self);
	}
}
