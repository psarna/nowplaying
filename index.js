import Database from 'libsql';
import { statSync, utimesSync } from 'fs';
import console from 'console';

const DB_PATH = '/tmp/status.db';
const LIBSQL_SYNC_URL = process.env.LIBSQL_SYNC_URL || '';
const LIBSQL_AUTH_TOKEN = process.env.LIBSQL_AUTH_TOKEN || '';

function formatLastSeen(ts) {
    const now = Date.now() / 1000;
    const delta = now - ts;
    if (delta < 60) {
        return `${Math.round(delta)} seconds ago`;
    } else if (delta < 3600) {
        return `${Math.floor(delta / 60)} minutes ago`;
    } else if (delta < 86400) {
        return `${Math.floor(delta / 3600)} hours ago`;
    } else {
        return `${Math.floor(delta / 86400)} days ago`;
    }
}

function dbFresh() {
    try {
        const stats = statSync(DB_PATH);
        const now = Date.now();
        const modified = stats.mtime.getTime();
        const differenceInSeconds = (now - modified) / 1000;
        console.log(`Last modified ${differenceInSeconds} seconds ago`);
        return differenceInSeconds < 30;
    } catch (_) {
        console.log("No DB file");
        return false;
    }
}

function update(db, who, playing) {
    db.exec("CREATE TABLE IF NOT EXISTS users(username TEXT PRIMARY KEY, last_seen INT, playing TEXT)");
    const now = Math.floor(Date.now() / 1000);
    const stmt = db.prepare("INSERT INTO users(username, last_seen, playing) VALUES (?, ?, ?) ON CONFLICT(username) DO UPDATE SET last_seen = excluded.last_seen, playing = excluded.playing");
    stmt.run(who, now, playing);
}

export async function handler(event) {
    const query = event.queryStringParameters || {};
    const who = query.name || 'anonymous';
    const playing = query.playing || '';

    let needs_sync = !dbFresh();

    const db = new Database(DB_PATH, {
        syncUrl: LIBSQL_SYNC_URL,
        authToken: LIBSQL_AUTH_TOKEN,
    });

    if (query.playing) {
        console.log(`Updating ${who} -> ${playing}`);
        update(db, who, playing);
        needs_sync |= true;
    }

    if (needs_sync) {
        console.log("Syncing DB");
        try {
            await db.sync();
            const now = new Date();
            utimesSync(DB_PATH, now, now);
        } catch (error) {
            console.error("Error syncing DB: ", error);
        }
    }

    let users;
    try {
        users = db.prepare("SELECT username, last_seen, playing FROM users").all().map(row => ({
            username: row.username,
            last_seen: formatLastSeen(row.last_seen),
            playing: row.playing,
        }));
    } catch (_) {
        users = [];
    }

    const message = {
        user: who,
        needed_sync: needs_sync,
        users: users,
    };

    return {
        statusCode: 200,
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(message),
    };
}