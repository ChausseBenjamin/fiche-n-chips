WITH RECURSIVE split(slug, rest, ord) AS (
		SELECT NULL, ? || '/', 0 UNION ALL
		SELECT substr(rest, 1, instr(rest, '/') - 1),
					 substr(rest, instr(rest, '/') + 1),
					 ord + 1
		FROM split WHERE rest <> '' AND instr(rest, '/') > 0
)
INSERT INTO routes (id, kind, hash, content)
SELECT slug, ?, ?, ? FROM split
WHERE ord > 0 AND slug <> '' AND NOT EXISTS
(SELECT 1 FROM routes AS e WHERE e.id = split.slug)
ORDER BY ord LIMIT 1
RETURNING id AS "id!: _", kind AS "kind!: _", created AS "created!: _"
