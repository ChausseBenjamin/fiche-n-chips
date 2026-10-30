SELECT id AS "id!: _", kind AS "kind!: _", created AS "created!: _"
FROM routes where kind != 0
AND created BETWEEN ? AND ?
