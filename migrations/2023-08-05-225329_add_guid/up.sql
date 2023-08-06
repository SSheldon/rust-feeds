ALTER TABLE item ADD guid VARCHAR;
ALTER TABLE item ADD CONSTRAINT item_feed_id_guid_key UNIQUE (feed_id, guid);
