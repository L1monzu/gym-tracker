//! Handles opening the SQLite database and keeping its schema up to date.
//!
//! When the app starts, we open (or create) the database file and run any
//! migrations that haven't been applied yet. This means a brand new install
//! and an existing one both end up with the same table structure, without
//! any manual setup steps for the user.

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
pub mod exercises;
pub mod cardio;
pub mod history;

/// Works out the right folder to store app data in, depending on which
/// platform we're running on.
///
/// Desktop platforms (Windows/macOS/Linux) have a well-established,
/// conventional per-user data folder that the `dirs` crate already knows
/// how to find. Android has no such convention — instead, every app must
/// ask the Android operating system directly for its own private storage
/// folder, which we do here via a small bit of Java interop.
pub fn app_data_dir() -> std::path::PathBuf {
    #[cfg(target_os = "android")]
    {
        android_files_dir()
    }

    #[cfg(not(target_os = "android"))]
    {
        dirs::data_dir().expect("could not find a data directory on this platform")
    }
}

/// Asks Android directly, via its own Java API, for this app's private
/// internal storage folder (equivalent to calling `context.getFilesDir()`
/// from Java/Kotlin code).
#[cfg(target_os = "android")]
fn android_files_dir() -> std::path::PathBuf {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }
        .expect("failed to get a handle to the Android Java VM");
    let mut env = vm
        .attach_current_thread()
        .expect("failed to attach the current thread to the Java VM");
    let activity = unsafe { jni::objects::JObject::from_raw(ctx.context().cast()) };

    let files_dir = env
        .call_method(&activity, "getFilesDir", "()Ljava/io/File;", &[])
        .expect("failed to call getFilesDir()")
        .l()
        .expect("getFilesDir() did not return an object");

    let path_string: jni::objects::JString = env
        .call_method(&files_dir, "getAbsolutePath", "()Ljava/lang/String;", &[])
        .expect("failed to call getAbsolutePath()")
        .l()
        .expect("getAbsolutePath() did not return an object")
        .into();

    let path: String = env
        .get_string(&path_string)
        .expect("failed to read the path string")
        .into();

    std::path::PathBuf::from(path)
}

/// Opens a connection pool to the app's SQLite database, creating the
/// database file and running any pending migrations if needed.
///
/// `db_path` is the full filesystem path where the database file should
/// live. On a real device, this will be a path inside the app's private
/// storage directory (wired up in a later step).
pub async fn init_db(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    // "sqlite://<path>?mode=rwc" tells sqlx: connect to a SQLite file at
    // this path, opening it for Read/Write, and Create it if it doesn't
    // already exist (that's what "rwc" stands for).
    let connection_string = format!("sqlite://{db_path}?mode=rwc");

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await?;

    // Applies any migration files in `migrations/` that haven't already
    // been run against this database. Safe to call every time the app
    // starts - already-applied migrations are simply skipped.
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}