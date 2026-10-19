DELETE FROM routes WHERE id = ? RETURNING id, kind, created_on
