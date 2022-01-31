#![allow(clippy::many_single_char_names)]

use serde::{
	self,
	de::{
		self,
		Visitor,
	},
	Deserialize,
	Deserializer,
	Serialize,
	Serializer,
};

use std::fmt;
use std::fmt::Debug;
use std::intrinsics::copy_nonoverlapping;

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub struct Uuid {
	a: u32,
	b: u16,
	c: u16,
	d: u64,
}

impl Uuid {
	pub fn new() -> Self {
		let mut bytes = [0u8; 16];
		getrandom::getrandom(&mut bytes)
			.unwrap_or_else(|err| panic!("Could not generate random bytes for uuid: {}", err));

		unsafe {
			let mut result = std::mem::zeroed();
			copy_nonoverlapping(
				bytes.as_ptr(),
				&mut result as *mut Uuid as *mut u8,
				bytes.len(),
			);
			result
		}
	}
}

impl Default for Uuid {
	fn default() -> Self {
		Self::new()
	}
}

impl From<&str> for Uuid {
	fn from(v: &str) -> Self {
		if !v.starts_with('{') || !v.ends_with('}') {
			panic!();
		}

		let v = v.strip_prefix('{').unwrap();
		let v = v.strip_suffix('}').unwrap();

		let values: Vec<&str> = v.split('-').collect();
		if values.len() != 5 {
			panic!();
		}

		let a: u32 = u32::from_str_radix(values[0], 16).unwrap();
		let b: u16 = u16::from_str_radix(values[1], 16).unwrap();
		let c: u16 = u16::from_str_radix(values[2], 16).unwrap();

		let d0: u16 = u16::from_str_radix(values[3], 16).unwrap();
		let d1: u64 = u64::from_str_radix(values[4], 16).unwrap();

		let d = ((d0 as u64) << 48) | d1;

		Self { a, b, c, d }
	}
}

impl Serialize for Uuid {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let d = self.d >> 48;
		let e = self.d & 0x0000FFFFFFFFFFFF;
		let mut out = format!(
			"{{{:#8x}-{:#4x}-{:#4x}-{:#4x}-{:#12x}}}",
			self.a, self.b, self.c, d, e
		);
		out.remove_matches("0x");
		serializer.serialize_str(&out)
	}
}

impl<'de> Deserialize<'de> for Uuid {
	#[allow(clippy::many_single_char_names)]
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct UUIDString;

		impl<'de> Visitor<'de> for UUIDString {
			type Value = Uuid;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter
					.write_str("an string assembled as such {00000000-0000-0000-0000-000000000000}")
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where
				E: de::Error,
			{
				if !v.starts_with('{') || !v.ends_with('}') {
					return Err(de::Error::invalid_type(de::Unexpected::Str(v), &self));
				}

				let v = v.strip_prefix('{').unwrap();
				let v = v.strip_suffix('}').unwrap();

				let values: Vec<&str> = v.split('-').collect();
				if values.len() != 5 {
					return Err(de::Error::invalid_type(de::Unexpected::Str(v), &self));
				}

				let a: u32 = u32::from_str_radix(values[0], 16)
					.map_err(|_| de::Error::invalid_type(de::Unexpected::Str(values[0]), &self))?;
				let b: u16 = u16::from_str_radix(values[1], 16)
					.map_err(|_| de::Error::invalid_type(de::Unexpected::Str(values[1]), &self))?;
				let c: u16 = u16::from_str_radix(values[2], 16)
					.map_err(|_| de::Error::invalid_type(de::Unexpected::Str(values[2]), &self))?;

				let d0: u16 = u16::from_str_radix(values[3], 16)
					.map_err(|_| de::Error::invalid_type(de::Unexpected::Str(values[3]), &self))?;
				let d1: u64 = u64::from_str_radix(values[4], 16)
					.map_err(|_| de::Error::invalid_type(de::Unexpected::Str(values[4]), &self))?;

				let d = ((d0 as u64) << 48) | d1;

				Ok(Self::Value { a, b, c, d })
			}
		}

		deserializer.deserialize_str(UUIDString)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn serialize() {
		let uuid = Uuid {
			a: 0x123456,
			b: 0x789A,
			c: 0xBCDE,
			d: 0xF0123456789ABCDE,
		};

		let serialized = serde::ron::to_string(&uuid).unwrap();
		assert_eq!(serialized, "\"{123456-789a-bcde-f012-3456789abcde}\"");
	}

	#[test]
	fn deserialize() {
		let uuid_string = "\"{123456-789a-bcde-f012-3456789abcde}\"";

		let deserialized: Uuid = serde::ron::from_str(uuid_string).unwrap();
		assert_eq!(
			deserialized,
			Uuid {
				a: 0x123456,
				b: 0x789A,
				c: 0xBCDE,
				d: 0xF0123456789ABCDE
			}
		);
	}
}

impl Debug for Uuid {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let serialized = serde::ron::to_string(self).unwrap();
		f.write_str(&serialized)
	}
}
