mod bearer;
mod origin;
mod token;

pub use bearer::*;
pub use origin::*;
pub use token::*;

pub trait UserSecurityPolicy {
	fn get_name(&self) -> String;
	fn get_description(&self) -> String;
	fn check(&self, username: &str, password: &str) -> Result<(), String>;
}
