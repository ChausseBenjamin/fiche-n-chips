CREATE TABLE routes (
	id TEXT PRIMARY KEY,   -- url path (after the root domain)
	kind INTEGER NOT NULL, -- redirect or pastebin or *reserved*
	hash INTEGER UNIQUE,      -- can be null for redirects (used to avoid duplicate pastes)
	content BLOB,          -- target url or raw paste data (compressed)
  created_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
