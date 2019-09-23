use rusqlite as sql;
use self::sql::OptionalExtension;
use proto;

pub struct Database {
	db: sql::Connection,
}

impl Database {
	pub fn new() -> sql::Result<Self> {
		Ok(Self {
			db: sql::Connection::open_with_flags("chorus_studio.db", sql::OpenFlags::SQLITE_OPEN_READ_WRITE)?,
		})
	}

	pub fn users_from_user_name_iter<'a>(&self, names: impl Iterator<Item = &'a str>) -> sql::Result<Vec<proto::User>> {
		let mut query = String::from("SELECT user.user_name FROM user WHERE user.user_name IN (");
		let names: Vec<_> = names.collect();

		for i in 0..names.len() {
			if i > 0 {
				query.push_str(", ");
			}
			query.push('?');
		}
		query.push(')');

		let mut stmt = self.db.prepare(&query)?;
		let iter = stmt.query_map(names.into_iter(), |row| {
			Ok(proto::User {
				user_name: row.get(0)?,
				activity: proto::UserActivity::Active,
			})
		})?.filter_map(Result::ok);
		Ok(iter.collect())
	}

	pub fn user_with_credentials(&self, email: &str, password_hashed: &[u8]) -> sql::Result<Option<proto::User>> {
		let mut stmt = self.db.prepare(r#"
			SELECT user.user_name FROM user
			WHERE user.email = :email AND user.password = :password
		"#)?;
		stmt.query_row_named(&[(":email", &email), (":password", &password_hashed)], |row| {
			Ok(proto::User {
				user_name: row.get(0)?,
				activity: proto::UserActivity::Active,
			})
		}).optional()
	}
	
	pub fn all_users(&self) -> sql::Result<Vec<proto::User>> {
		let mut stmt = self.db.prepare("select user.user_name from user")?;
		let iter = stmt.query_map(sql::NO_PARAMS, |row| {
			Ok(proto::User {
				user_name: row.get(0)?,
				activity: proto::UserActivity::Active,
			})
		})?.filter_map(Result::ok);
		Ok(iter.collect())
	}
}