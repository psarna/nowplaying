use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use libsql::{Connection, Database};

const DB_PATH: &str = "/tmp/status.db";

// Open the database in embedded replica mode that syncs with sync_url.
async fn open_db(
    sync_url: impl Into<String>,
    auth_token: impl Into<String>,
) -> libsql::Result<Database> {
    let sync_url = sync_url.into();
    let auth_token = auth_token.into();

    Database::open_with_remote_sync(DB_PATH, sync_url, auth_token).await
}

// Pretty print the last seen timestamp.
fn format_last_seen(ts: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let delta = now - ts;
    if delta < 60 {
        format!("{delta} seconds ago")
    } else if delta < 3600 {
        format!("{} minutes ago", delta / 60)
    } else if delta < 86400 {
        format!("{} hours ago", delta / 3600)
    } else {
        format!("{} days ago", delta / 86400)
    }
}

// A database is considered fresh if it exists and was last updated
// no more than 30 seconds ago.
fn db_fresh() -> bool {
    let now = std::time::SystemTime::now();
    match std::fs::metadata(DB_PATH) {
        Ok(metadata) => {
            let modified = match metadata.modified() {
                Ok(modified) => now.duration_since(modified).unwrap_or_default(),
                Err(_) => return false,
            };
            tracing::info!("Last modified {} seconds ago", modified.as_secs());
            modified.as_secs() < 30
        }
        Err(_) => false,
    }
}

async fn update(conn: &Connection, who: &str, playing: &str) -> libsql::Result<()> {
    let now = chrono::Utc::now().timestamp();
    // TODO: both should be sent in a single batch
    conn.execute("create table if not exists users(username text primary key, last_seen int, playing text) without rowid", ()).await?;
    conn.execute(
        "INSERT INTO users VALUES (?, ?, ?)
        ON CONFLICT(username) DO UPDATE SET last_seen = ?, playing = ?",
        (who, now, playing, now, playing),
    )
    .await
    .map(|_| ())
}

// Main entrypoint to the lambda logic
async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let who = event
        .query_string_parameters_ref()
        .and_then(|params| params.first("name"))
        .unwrap_or("anonymous");
    let playing = event
        .query_string_parameters_ref()
        .and_then(|params| params.first("playing"));

    let needs_sync = !db_fresh();

    let sync_url = std::env::var("LIBSQL_SYNC_URL").map_err(Box::new)?;
    let auth_token = std::env::var("LIBSQL_AUTH_TOKEN").unwrap_or("".to_string());
    let db = open_db(sync_url, auth_token).await.map_err(Box::new)?;

    if needs_sync {
        tracing::info!("Syncing database with remote counterpart");
        db.sync().await?;
    } else {
        tracing::info!("Database exists, no sync needed")
    }
    let conn = db.connect()?;

    let mut message = format!("Hi {who}!\n");

    if let Some(playing) = playing {
        message += &format!("Enjoying {playing} huh\n");
        update(&conn, who, playing).await?;
    }

    let mut rows = conn
        .query("SELECT * FROM users WHERE username != ?", &[who])
        .await?;

    if needs_sync {
        message += "\t(the database was freshly synced before serving the request)\n";
    } else {
        message += "\t(serving cached results)\n"
    }

    while let Ok(Some(row)) = rows.next() {
        let user = row.get_str(0)?;
        let last_seen = row.get::<i64>(1)?;
        let last_seen = format_last_seen(last_seen);
        let playing = row.get_str(2)?;
        message += &format!("User {user},\tlast seen {last_seen},\tplaying {playing}\n",);
    }

    // Return something that implements IntoResponse.
    // It will be serialized to the right response event automatically by the runtime
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(message.into())
        .map_err(Box::new)?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
