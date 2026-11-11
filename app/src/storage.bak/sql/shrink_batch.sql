DELETE FROM routes WHERE kind != 0
  AND id IN (
    SELECT id FROM routes
    WHERE kind != 0
    ORDER BY created_on ASC
    LIMIT 10
  )
RETURNING id, kind, created_on
