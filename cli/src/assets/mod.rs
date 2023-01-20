pub const LOGO: &[u8] = include_bytes!("./logo.png");
pub const MOST_USED_PASSWORDS: &[u8] = include_bytes!("./most_used_passwords.txt");
pub const REMOTE_STORAGE: &[u8] = include_bytes!("./remoteStorage.svg");
pub const SERVER_INDEX: &str = include_str!("./index.html");
pub const EASY_TO_GUESS_USERS: &[&str] = &[
	"",
	"admin",
	"administrator",
	"root",
	"main",
	"user",
	"username",
];