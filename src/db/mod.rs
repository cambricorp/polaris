use core::ops::Deref;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_migrations;
use error_chain::bail;
use log::info;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::errors::*;

mod schema;

pub use self::schema::*;

#[allow(dead_code)]
const DB_MIGRATIONS_PATH: &str = "migrations";
embed_migrations!("migrations");

pub trait ConnectionSource {
	fn get_connection(&self) -> MutexGuard<'_, SqliteConnection>;
	fn get_connection_mutex(&self) -> Arc<Mutex<SqliteConnection>>;
}

pub struct DB {
	connection: Arc<Mutex<SqliteConnection>>,
}

impl DB {
	pub fn new(path: &Path) -> Result<DB> {
		info!("Database file path: {}", path.to_string_lossy());
		let connection = Arc::new(Mutex::new(SqliteConnection::establish(
			&path.to_string_lossy(),
		)?));
		let db = DB {
			connection: connection.clone(),
		};
		db.init()?;
		Ok(db)
	}

	fn init(&self) -> Result<()> {
		{
			let connection = self.connection.lock().unwrap();
			connection.execute("PRAGMA synchronous = NORMAL")?;
		}
		self.migrate_up()?;
		Ok(())
	}

	#[allow(dead_code)]
	fn migrate_down(&self) -> Result<()> {
		let connection = self.connection.lock().unwrap();
		let connection = connection.deref();
		loop {
			match diesel_migrations::revert_latest_migration_in_directory(
				connection,
				Path::new(DB_MIGRATIONS_PATH),
			) {
				Ok(_) => (),
				Err(diesel_migrations::RunMigrationsError::MigrationError(
					diesel_migrations::MigrationError::NoMigrationRun,
				)) => break,
				Err(e) => bail!(e),
			}
		}
		Ok(())
	}

	fn migrate_up(&self) -> Result<()> {
		let connection = self.connection.lock().unwrap();
		let connection = connection.deref();
		embedded_migrations::run(connection)?;
		Ok(())
	}
}

impl ConnectionSource for DB {
	fn get_connection(&self) -> MutexGuard<'_, SqliteConnection> {
		self.connection.lock().unwrap()
	}

	fn get_connection_mutex(&self) -> Arc<Mutex<SqliteConnection>> {
		self.connection.clone()
	}
}

pub fn _get_test_db(name: &str) -> DB {
	use crate::config;
	let config_path = Path::new("test/config.toml");
	let config = config::parse_toml_file(&config_path).unwrap();

	let mut db_path = PathBuf::new();
	db_path.push("test");
	db_path.push(name);
	if db_path.exists() {
		fs::remove_file(&db_path).unwrap();
	}

	let db = DB::new(&db_path).unwrap();
	config::overwrite(&db, &config).unwrap();
	db
}

#[test]
fn test_migrations_up() {
	_get_test_db("migrations_up.sqlite");
}

#[test]
fn test_migrations_down() {
	let db = _get_test_db("migrations_down.sqlite");
	db.migrate_down().unwrap();
	db.migrate_up().unwrap();
}
