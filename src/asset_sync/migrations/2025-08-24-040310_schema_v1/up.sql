-- Lookup tables
CREATE TABLE provider (
    -- normalized lowercase key, e.g. 'alpaca'
    code TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE asset_class (
    code TEXT PRIMARY KEY NOT NULL        -- e.g. 'us_equity','futures'
);

CREATE TABLE provider_asset_class (
    provider_code TEXT NOT NULL,
    asset_class_code TEXT NOT NULL,
    PRIMARY KEY (provider_code, asset_class_code),
    FOREIGN KEY (provider_code) REFERENCES provider (code)
    ON DELETE RESTRICT ON UPDATE CASCADE,
    FOREIGN KEY (asset_class_code) REFERENCES asset_class (code)
    ON DELETE RESTRICT ON UPDATE CASCADE
);

CREATE TABLE provider_symbol_map (
    provider_code TEXT NOT NULL,
    asset_class_code TEXT NOT NULL,
    canonical_symbol TEXT NOT NULL,   -- our 'AAPL','ES', etc.
    remote_symbol TEXT NOT NULL,   -- provider-specific, e.g. 'AAPL','ESZ5'
    PRIMARY KEY (provider_code, asset_class_code, canonical_symbol),
    UNIQUE (provider_code, asset_class_code, remote_symbol),
    FOREIGN KEY (provider_code, asset_class_code)
    REFERENCES provider_asset_class (provider_code, asset_class_code)
    ON DELETE RESTRICT ON UPDATE CASCADE
);

-- 1) identity + desired coverage + progress (one row per stream)
CREATE TABLE asset_manifest (
    id INTEGER PRIMARY KEY NOT NULL,
    symbol TEXT NOT NULL,
    provider_code TEXT NOT NULL,
    asset_class_code TEXT NOT NULL,         -- e.g. "us_equity","futures"
    timeframe_amount INTEGER NOT NULL,
    timeframe_unit TEXT NOT NULL,         -- e.g. "Minute","Day"
    desired_start TEXT NOT NULL,         -- RFC3339 UTC
    desired_end TEXT,                  -- NULL=open-ended keep-fresh
    watermark TEXT,                  -- RFC3339 UTC contiguous progress
    last_error TEXT,

    -- timestamps: RFC3339 UTC with millisecond precision
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    -- monotonic revision counter (bumps on any UPDATE)
    update_rev INTEGER NOT NULL DEFAULT 0,
    CHECK (
        (timeframe_unit = 'Minute' AND timeframe_amount BETWEEN 1 AND 59)
        OR (timeframe_unit = 'Hour' AND timeframe_amount BETWEEN 1 AND 23)
        OR (timeframe_unit = 'Day' AND timeframe_amount = 1)
        OR (timeframe_unit = 'Week' AND timeframe_amount = 1)
        OR (
            timeframe_unit = 'Month'
            AND timeframe_amount IN (1, 2, 3, 4, 6, 12)
        )
    ),
    UNIQUE (
        symbol,
        provider_code,
        asset_class_code,
        timeframe_amount,
        timeframe_unit
    ),
    FOREIGN KEY (provider_code, asset_class_code)
    REFERENCES provider_asset_class (provider_code, asset_class_code)
    ON DELETE RESTRICT ON UPDATE CASCADE
);

-- trigger: set updated_at precisely on every update
DROP TRIGGER IF EXISTS trg_asset_manifest_updated;

-- Monotonic "touch" trigger
-- Notes:
--  * The WHERE clause prevents infinite recursion: the second firing sees
--     updated_at changed and does nothing.
--  * 'now' can equal OLD.updated_at within the same sqlite3_step(), so 
--     we add +0.001 seconds.
CREATE TRIGGER trg_asset_manifest_updated
AFTER UPDATE ON asset_manifest
FOR EACH ROW
WHEN NEW.updated_at = OLD.updated_at
BEGIN
UPDATE asset_manifest
SET
    updated_at = CASE
        WHEN julianday('now') > julianday(OLD.updated_at)
            THEN strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
        ELSE strftime(
            '%Y-%m-%dT%H:%M:%fZ',
            datetime(OLD.updated_at, '+0.001 seconds')
        )
    END,
    update_rev = OLD.update_rev + 1
WHERE
    id = old.id
    AND updated_at = OLD.updated_at;
END;

-- 2) roaring bitmap per manifest = “what we have” (with OCC)
CREATE TABLE asset_coverage_bitmap (
    id INTEGER PRIMARY KEY NOT NULL,
    manifest_id INTEGER NOT NULL,
    bitmap BLOB NOT NULL,          -- roaring serialized bytes
    version INTEGER NOT NULL DEFAULT 0, -- optimistic concurrency
    -- FK: if a manifest goes away, clean up the blob
    FOREIGN KEY (manifest_id) REFERENCES asset_manifest (id)
    ON DELETE CASCADE ON UPDATE CASCADE,
    UNIQUE (manifest_id)
);

-- 3) durable backlog of requested backfills (operational work items)
CREATE TABLE asset_gaps (
    id INTEGER PRIMARY KEY NOT NULL,
    manifest_id INTEGER NOT NULL,
    start_ts TEXT NOT NULL,        -- RFC3339 UTC inclusive
    end_ts TEXT NOT NULL,        -- RFC3339 UTC inclusive
    state TEXT NOT NULL DEFAULT 'queued', -- queued|leased|done|failed
    lease_owner TEXT,
    lease_expires_at TEXT,
    CHECK (state IN ('queued', 'leased', 'done', 'failed')),
    -- FK: gaps are tied to a manifest (choose policy below)
    FOREIGN KEY (manifest_id) REFERENCES asset_manifest (id)
    ON DELETE CASCADE ON UPDATE CASCADE,
    UNIQUE (manifest_id, start_ts, end_ts)
);

-- helpful indexes for leasing scans
CREATE INDEX gaps_manifest_state_expiry
ON asset_gaps (manifest_id, state, lease_expires_at);
CREATE INDEX gaps_manifest_start
ON asset_gaps (manifest_id, start_ts);
