//! Handles opening the SQLite database and keeping its schema up to date.
//!
//! When the app starts, we open (or create) the database file and run any
//! migrations that haven't been applied yet. This means a brand new install
//! and an existing one both end up with the same table structure, without
//! any manual setup steps for the user.

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

pub mod cardio;
pub mod exercises;
pub mod export;
pub mod history;
pub mod templates;

/// Works out the right folder to store app data in, depending on which
/// platform we're running on.
///
/// Desktop platforms (Windows/macOS/Linux) have a well-established,
/// conventional per-user data folder that the `dirs` crate already knows
/// how to find. Android has no such convention, instead, every app must
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

/// Returns a folder suitable for files the user might want to find and
/// share manually, like an Excel export. On Android this is the app's
/// external files folder, browsable with a normal file manager. On
/// desktop, it's the same per-user data folder used for everything else.
pub fn shareable_files_dir() -> std::path::PathBuf {
    #[cfg(target_os = "android")]
    {
        android_external_files_dir()
    }

    #[cfg(not(target_os = "android"))]
    {
        app_data_dir()
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

/// Asks Android for this app's external files folder. Kept around as a
/// building block, note that this folder is hidden from normal file
/// manager apps on modern Android, `save_to_downloads` is the function
/// that actually puts a file somewhere the user can find it.
#[cfg(target_os = "android")]
fn android_external_files_dir() -> std::path::PathBuf {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }
        .expect("failed to get a handle to the Android Java VM");
    let mut env = vm
        .attach_current_thread()
        .expect("failed to attach the current thread to the Java VM");
    let activity = unsafe { jni::objects::JObject::from_raw(ctx.context().cast()) };

    let null_type = jni::objects::JObject::null();
    let files_dir = env
        .call_method(
            &activity,
            "getExternalFilesDir",
            "(Ljava/lang/String;)Ljava/io/File;",
            &[(&null_type).into()],
        )
        .expect("failed to call getExternalFilesDir()")
        .l()
        .expect("getExternalFilesDir() did not return an object");

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

/// Writes bytes into the public Downloads folder via Android's
/// MediaStore API. This is the only reliable way to put a file
/// somewhere the Files app and other apps like Drive can see it, the
/// app's own external files folder is hidden from file managers on
/// modern Android.
#[cfg(target_os = "android")]
pub fn save_to_downloads(file_name: &str, bytes: &[u8]) -> Result<(), String> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }
        .map_err(|e| format!("failed to get JVM: {e}"))?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|e| format!("failed to attach thread: {e}"))?;
    let activity = unsafe { jni::objects::JObject::from_raw(ctx.context().cast()) };

    let content_values = env
        .new_object("android/content/ContentValues", "()V", &[])
        .map_err(|e| format!("failed to create ContentValues: {e}"))?;

    let display_name_key = env.new_string("_display_name").unwrap();
    let display_name_value = env.new_string(file_name).unwrap();
    env.call_method(
        &content_values,
        "put",
        "(Ljava/lang/String;Ljava/lang/String;)V",
        &[(&display_name_key).into(), (&display_name_value).into()],
    )
    .map_err(|e| format!("failed to set display name: {e}"))?;

    let mime_key = env.new_string("mime_type").unwrap();
    let mime_value = env
        .new_string("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        .unwrap();
    env.call_method(
        &content_values,
        "put",
        "(Ljava/lang/String;Ljava/lang/String;)V",
        &[(&mime_key).into(), (&mime_value).into()],
    )
    .map_err(|e| format!("failed to set mime type: {e}"))?;

    let resolver = env
        .call_method(
            &activity,
            "getContentResolver",
            "()Landroid/content/ContentResolver;",
            &[],
        )
        .map_err(|e| format!("failed to get content resolver: {e}"))?
        .l()
        .map_err(|e| format!("content resolver was not an object: {e}"))?;

    let downloads_uri_class = env
        .find_class("android/provider/MediaStore$Downloads")
        .map_err(|e| format!("failed to find MediaStore.Downloads: {e}"))?;
    let external_uri = env
        .get_static_field(
            downloads_uri_class,
            "EXTERNAL_CONTENT_URI",
            "Landroid/net/Uri;",
        )
        .map_err(|e| format!("failed to get EXTERNAL_CONTENT_URI: {e}"))?
        .l()
        .map_err(|e| format!("EXTERNAL_CONTENT_URI was not an object: {e}"))?;

    let inserted_uri = env
        .call_method(
            &resolver,
            "insert",
            "(Landroid/net/Uri;Landroid/content/ContentValues;)Landroid/net/Uri;",
            &[(&external_uri).into(), (&content_values).into()],
        )
        .map_err(|e| format!("failed to insert into MediaStore: {e}"))?
        .l()
        .map_err(|e| format!("insert did not return a Uri: {e}"))?;

    if inserted_uri.is_null() {
        return Err("MediaStore insert returned null".to_string());
    }

    let output_stream = env
        .call_method(
            &resolver,
            "openOutputStream",
            "(Landroid/net/Uri;)Ljava/io/OutputStream;",
            &[(&inserted_uri).into()],
        )
        .map_err(|e| format!("failed to open output stream: {e}"))?
        .l()
        .map_err(|e| format!("openOutputStream did not return an object: {e}"))?;

    let byte_array = env
        .byte_array_from_slice(bytes)
        .map_err(|e| format!("failed to build byte array: {e}"))?;

    env.call_method(&output_stream, "write", "([B)V", &[(&byte_array).into()])
        .map_err(|e| format!("failed to write bytes: {e}"))?;

    env.call_method(&output_stream, "close", "()V", &[])
        .map_err(|e| format!("failed to close stream: {e}"))?;

    Ok(())
}

/// Opens a connection pool to the app's SQLite database, creating the
/// database file and running any pending migrations if needed.
///
/// `db_path` is the full filesystem path where the database file should
/// live. On a real device, this will be a path inside the app's private
/// storage directory.
pub async fn init_db(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    // "sqlite://<path>?mode=rwc" tells sqlx, connect to a SQLite file at
    // this path, opening it for Read/Write, and Create it if it doesn't
    // already exist (that's what "rwc" stands for).
    let connection_string = format!("sqlite://{db_path}?mode=rwc");

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await?;

    // Applies any migration files in `migrations/` that haven't already
    // been run against this database. Safe to call every time the app
    // starts, already-applied migrations are simply skipped.
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}