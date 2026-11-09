DELETE FROM routes
WHERE kind != 0 AND id IN (
    SELECT id FROM routes WHERE kind != 0
    ORDER BY created ASC
    LIMIT ?
  )
