-- 1) identity + desired coverage + progress (one row per stream)
CREATE TABLE asset_manifest (
    id INTEGER PRIMARY KEY,
    symbol TEXT NOT NULL,
    provider TEXT NOT NULL,
    asset_class TEXT NOT NULL,         -- e.g. "us_equity","futures"
    timeframe_amount INTEGER NOT NULL,
    timeframe_unit TEXT NOT NULL,         -- e.g. "Minute","Day"
    desired_start TEXT NOT NULL,         -- RFC3339 UTC
    desired_end TEXT,                  -- NULL=open-ended keep-fresh
    watermark TEXT,                  -- RFC3339 UTC contiguous progress
    last_error TEXT,
    created_at TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    updated_at TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    CHECK (timeframe_unit IN ('Minute', 'Day')),
    UNIQUE (symbol, provider, asset_class, timeframe_amount, timeframe_unit)
);

-- keep updated_at fresh on any update
CREATE TRIGGER trg_asset_manifest_updated
AFTER UPDATE ON asset_manifest
FOR EACH ROW
BEGIN
UPDATE asset_manifest SET updated_at = CURRENT_TIMESTAMP
WHERE id = old.id;
END;

-- 2) roaring bitmap per manifest = “what we have” (with OCC)
CREATE TABLE asset_coverage_bitmap (
    id INTEGER PRIMARY KEY,
    manifest_id INTEGER NOT NULL,
    bitmap BLOB NOT NULL,          -- roaring serialized bytes
    version INTEGER NOT NULL DEFAULT 0, -- optimistic concurrency
    UNIQUE (manifest_id)
);

-- 3) durable backlog of requested backfills (operational work items)
CREATE TABLE asset_gaps (
    id INTEGER PRIMARY KEY,
    manifest_id INTEGER NOT NULL,
    start_ts TEXT NOT NULL,        -- RFC3339 UTC inclusive
    end_ts TEXT NOT NULL,        -- RFC3339 UTC inclusive
    state TEXT NOT NULL DEFAULT 'queued', -- queued|leased|done|failed
    lease_owner TEXT,
    lease_expires_at TEXT,
    CHECK (state IN ('queued', 'leased', 'done', 'failed')),
    UNIQUE (manifest_id, start_ts, end_ts)
);

-- helpful indexes for leasing scans
CREATE INDEX gaps_manifest_state_expiry
ON asset_gaps (manifest_id, state, lease_expires_at);
CREATE INDEX gaps_manifest_start
ON asset_gaps (manifest_id, start_ts);
