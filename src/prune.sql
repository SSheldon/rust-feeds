DELETE FROM item WHERE item.is_read AND NOT item.is_saved AND NOT item.id IN (
    SELECT unnest(latest_items.ids) FROM (
        SELECT feed.id, ARRAY(
            SELECT item.id FROM item WHERE item.feed_id=feed.id ORDER BY item.id DESC LIMIT 10
        ) FROM feed GROUP BY feed.id
    ) as latest_items(feed_id, ids)
);
