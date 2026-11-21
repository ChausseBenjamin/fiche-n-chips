DELETE FROM routes WHERE id = ? RETURNING
id AS "id!: _",
kind AS "kind!: _",
created AS "created!: _"

