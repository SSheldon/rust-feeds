ALTER TABLE item DROP CONSTRAINT item_url_guid_not_null;
ALTER TABLE item ALTER COLUMN url SET NOT NULL;
