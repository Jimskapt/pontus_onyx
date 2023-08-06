pub const LOGO: &[u8] = include_bytes!("./logo.png");
pub const MOST_USED_PASSWORDS: &[u8] = include_bytes!("./most_used_passwords.txt");
pub const REMOTE_STORAGE: &[u8] = include_bytes!("./remoteStorage.svg");
pub const SERVER_INDEX: &str = include_str!("./index.html");
pub const ADMIN_UI_INDEX: &str = include_str!("./admin/index.html");
pub const ADMIN_UI_USERS: &str = include_str!("./admin/users.html");
pub const ADMIN_UI_USER: &str = include_str!("./admin/user.html");
pub const ADMIN_UI_SETTINGS: &str = include_str!("./admin/settings.html");
pub const SERVER_OAUTH: &str = include_str!("./oauth.html");
pub const EASY_TO_GUESS_USERS: &[&str] = &[
	"",
	"admin",
	"administrator",
	"root",
	"main",
	"user",
	"username",
];
