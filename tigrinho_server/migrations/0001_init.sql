-- 2025-10-26: initial schema
CREATE TABLE IF NOT EXISTS params (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    server_seed TEXT NOT NULL,
    server_seed_hash TEXT NOT NULL,
    rtp_target REAL NOT NULL,
    paytable_json TEXT NOT NULL,
    nonce INTEGER NOT NULL
);

INSERT OR IGNORE INTO params (id, server_seed, server_seed_hash, rtp_target, paytable_json, nonce)
VALUES (1, 'dev-secret-seed', '0', 0.95, '[]', 0);

CREATE TABLE IF NOT EXISTS spins (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts TEXT NOT NULL,
    client_seed TEXT NOT NULL,
    nonce INTEGER NOT NULL,
    server_seed_hash TEXT NOT NULL,
    result_reels_json TEXT NOT NULL,
    payout REAL NOT NULL
);

-- enforce append-only for spins
CREATE TRIGGER IF NOT EXISTS spins_no_update
BEFORE UPDATE ON spins
BEGIN
    SELECT RAISE(ABORT, 'spins is append-only');
END;

CREATE TRIGGER IF NOT EXISTS spins_no_delete
BEFORE DELETE ON spins
BEGIN
    SELECT RAISE(ABORT, 'spins is append-only');
END;
