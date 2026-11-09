WITH RECURSIVE seq(created) AS (
    SELECT 1
    UNION ALL
    SELECT created + 1 FROM seq WHERE created < ?
)
INSERT INTO routes (id, kind, hash, content, created)
SELECT 'paste-' || created, 2, NULL, ?, created FROM seq
