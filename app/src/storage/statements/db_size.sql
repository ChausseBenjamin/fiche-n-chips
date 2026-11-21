SELECT
    CAST(page_count     AS INTEGER) AS "page_count!: i64",
    CAST(freelist_count AS INTEGER) AS "freelist_count!: i64"
FROM pragma_page_count(), pragma_freelist_count()
