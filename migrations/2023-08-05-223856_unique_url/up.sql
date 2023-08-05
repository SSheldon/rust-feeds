ALTER TABLE item ADD CONSTRAINT item_feed_id_url_key UNIQUE (feed_id, url);
