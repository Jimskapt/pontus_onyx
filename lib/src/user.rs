use std::collections::BTreeMap;

#[derive(zeroize::Zeroize, zeroize::ZeroizeOnDrop, serde::Serialize, serde::Deserialize)]
pub struct User {
	#[zeroize(skip)]
	pub username: String,
	pub password: String,
	#[zeroize(skip)]
	pub tokens: BTreeMap<crate::security::Token, crate::security::TokenMetadata>,
	#[zeroize(skip)]
	pub special_roles: Vec<UserAdminRole>,
	#[zeroize(skip)]
	pub admin_tokens: BTreeMap<crate::security::Token, crate::security::TokenMetadata>,
}

impl serde_encrypt::traits::SerdeEncryptSharedKey for User {
	type S = serde_encrypt::serialize::impls::BincodeSerializer<Self>;
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, serde::Serialize, serde::Deserialize)]
pub enum UserAdminRole {
	ManageUsers,
	ManageServerSettings,
	ReadLogs,
} // if adding new role, do not forget to add it to `ALL_ROLES`
impl std::fmt::Display for UserAdminRole {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::ManageUsers => f.write_str("manage users"),
			Self::ManageServerSettings => f.write_str("manage server settings"),
			Self::ReadLogs => f.write_str("read logs"),
		}
	}
}

pub const ALL_ROLES: &[UserAdminRole] = &[
	UserAdminRole::ManageUsers,
	UserAdminRole::ManageServerSettings,
	UserAdminRole::ReadLogs,
];

#[derive(Clone)]
pub struct UserMetadata {
	pub username: String,
	pub special_roles: Vec<UserAdminRole>,
}
