DELETE FROM routes WHERE kind != 0 AND created_on < CAST(strftime('%s', 'now') AS INTEGER) - ? RETURNING id, kind, created_on
