#[derive(PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Content(Vec<u8>);

impl Content {
	pub fn into_inner(&self) -> &[u8] {
		&self.0
	}
}

impl<T: AsRef<[u8]>> From<T> for Content {
	fn from(data: T) -> Self {
		Self(data.as_ref().to_vec())
	}
}

static BYTES_TO_DEBUG: usize = 3;
impl std::fmt::Debug for Content {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		if self.0.len() > 2 * BYTES_TO_DEBUG {
			f.write_fmt(format_args!(
				"{:?}[hidden]{:?}",
				&self.0[..BYTES_TO_DEBUG],
				&self.0[(self.0.len() - BYTES_TO_DEBUG)..]
			))
		} else {
			f.write_fmt(format_args!("{:?}", self.0))
		}
	}
}
