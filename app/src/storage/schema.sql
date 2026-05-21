CREATE TABLE routes (
	id TEXT PRIMARY KEY,      -- url path (after the root domain)
	kind    INTEGER NOT NULL, -- redirect or pastebin or *reserved*
	hash    INTEGER,          -- can be null for redirects (used to avoid duplicate pastes)
	                          -- it is unlikely but possible different pastes have
														-- the same hash (not unique), in which case every
														-- entry will get checked individually.
	content BLOB,             -- target url or raw paste data (compressed)
  created_on TIMESTAMP NOT NULL DEFAULT (strftime('%s', 'now'))
);
