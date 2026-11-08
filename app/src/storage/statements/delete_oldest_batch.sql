DELETE FROM routes
WHERE kind != 0 AND id IN (
    SELECT id FROM routes WHERE kind != 0
    ORDER BY created ASC
    LIMIT ?
  )
RETURNING id AS "id!: _", kind AS "kind!: _", created AS "created!: _"
