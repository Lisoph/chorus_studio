use sqlite as sql;
use proto;

pub type Fetched<T> = Result<T, ()>;

trait ValueExt {
	fn fetch_string(&self) -> Fetched<&String>;
	fn fetch_opt_string(&self) -> Fetched<Option<&String>>;
}

impl ValueExt for sql::Value {
	fn fetch_string(&self) -> Fetched<&String> {
		if let sql::Value::String(s) = self {
			Ok(s)
		} else {
			Err(())
		}
	}

	fn fetch_opt_string(&self) -> Fetched<Option<&String>> {
		match self {
			sql::Value::String(s) => Ok(Some(s)),
			sql::Value::Null => Ok(None),
			_ => Err(()),
		}
	}
}

pub struct Database {
	db: sql::Connection,
}

impl Database {
	pub fn new() -> sql::Result<Self> {
		Ok(Self {
			db: sql::Connection::open("file:chorus_studio.db")?,
		})
	}

	pub fn users_from_user_name_iter(&self, names: impl Iterator<Item = impl AsRef<str>>) -> sql::Result<UserIter> {
		let mut query = String::from("SELECT user.user_name FROM user WHERE user.user_name IN (");
		let values: Vec<_> = names.map(|e| sql::Value::String(e.as_ref().to_owned())).collect();

		for i in 0..values.len() {
			if i > 0 {
				query.push_str(", ");
			}
			query.push('?');
		}
		query.push(')');

		let mut cursor = self.db.prepare(query)?.cursor();
		cursor.bind(&values)?;

		Ok(UserIter {
			cursor,
		})
	}

	pub fn user_with_credentials(&self, email: String, password_hashed: Vec<u8>) -> sql::Result<Option<Fetched<proto::User>>> {
		let mut cursor = self.db.prepare(r#"
			SELECT user.user_name FROM user
			WHERE user.email = ? AND user.password = ?
		"#)?.cursor();
		cursor.bind(&[sql::Value::String(email), sql::Value::Binary(password_hashed)])?;
		Ok((UserIter {
			cursor,
		}).next())
	}
}

pub struct UserIter<'a> {
	cursor: sql::Cursor<'a>,
}

impl<'a> Iterator for UserIter<'a> {
	type Item = Fetched<proto::User>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.cursor.next() {
			Ok(next) => next.map(|row| {
				Ok(proto::User {
					user_name: row[0].fetch_string()?.to_owned(),
					activity: proto::UserActivity::Active,
				})
			}),
			Err(_) => None,
		}
	}
}