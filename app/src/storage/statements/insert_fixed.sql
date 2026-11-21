INSERT INTO routes (id, kind, hash, content) VALUES (?, ?, ?, ?)
ON CONFLICT(id) DO NOTHING
RETURNING id AS "id!: _", kind AS "kind!: _", created AS "created!: _"
