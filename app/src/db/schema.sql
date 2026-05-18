CREATE TABLE routes (
	id TEXT PRIMARY KEY,   -- endpoint
	kind INTEGER NOT NULL, -- redirect or pastebin
	content TEXT NOT NULL, -- target url or raw text
  created_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
);
