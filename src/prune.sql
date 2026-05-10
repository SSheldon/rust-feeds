WITH numbered_item_stats AS (
    SELECT
        id,
        published,
        feed_id,
        row_number() OVER (PARTITION BY feed_id ORDER BY id DESC) AS feed_num
    FROM item
), feed_latest_stats AS (
    SELECT
        feed_id,
        min(id) AS min_id,
        min(published) AS min_published
    FROM numbered_item_stats
    WHERE feed_num <= 10
    GROUP BY feed_id
)
DELETE FROM item
WHERE
    is_read AND
    NOT is_saved AND
    id < (SELECT min_id FROM feed_latest_stats WHERE feed_id=item.feed_id) AND
    published < (SELECT min_published FROM feed_latest_stats where feed_id=item.feed_id);
