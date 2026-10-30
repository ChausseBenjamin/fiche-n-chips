CREATE TABLE routes (
    id         TEXT    PRIMARY KEY,  -- url path (after the root domain)
    kind       INTEGER NOT NULL,     -- redirect or pastebin or *reserved*
    hash       INTEGER,              -- can be null for redirects (used to avoid duplicate pastes)
                                     -- it is unlikely but possible different pastes have
                                     -- the same hash (not unique), in which case every
                                     -- entry will get checked individually.
    content    BLOB    DEFAULT NULL, -- target url or raw paste data (compressed)
    created_on INTEGER NOT NULL DEFAULT (strftime('%s', 'now')), -- unix timestamp
    CHECK (kind IN (0, 1, 2))        -- reserved, redirect, paste
);

CREATE INDEX idx_routes_hash
    ON routes (hash)
    WHERE hash IS NOT NULL; -- only index rows that have a hash (pastes)

CREATE INDEX idx_routes_cleanup
    ON routes (kind, created_on)
    WHERE kind != 0; -- reserved routes are never expired
