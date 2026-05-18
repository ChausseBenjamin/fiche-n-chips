CREATE TABLE routes (
	id TEXT PRIMARY KEY,          -- Full hash (or redirect url)
	slug_len INTEGER PRIMARY KEY, -- Nth first chars from the hash used as a url path ( len(id) for redirets )
	kind INTEGER NOT NULL, -- redirect or pastebin
	content TEXT NOT NULL, -- target url or raw text
  created_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
);
