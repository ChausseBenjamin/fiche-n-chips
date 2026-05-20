CREATE TABLE routes (
	id TEXT PRIMARY KEY,   -- url path (after the root domain)
	hash TEXT,             -- can be null for redirects
	kind INTEGER NOT NULL, -- redirect or pastebin or *reserved*
	content BLOB NOT NULL, -- target url or raw text (compressed)
  created_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
);
