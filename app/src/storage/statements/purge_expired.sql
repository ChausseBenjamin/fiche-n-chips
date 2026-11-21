DELETE FROM routes WHERE kind != 0 AND created < CAST(strftime('%s','now') AS INTEGER) - ?
RETURNING id AS "id!: _", kind AS "kind!: _", created AS "created!: _"
