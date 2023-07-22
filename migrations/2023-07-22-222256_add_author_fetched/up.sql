ALTER TABLE item ADD author VARCHAR;
ALTER TABLE item ADD fetched TIMESTAMP;
UPDATE item SET fetched = published;
ALTER TABLE item ALTER COLUMN fetched SET NOT NULL;
ALTER TABLE item ALTER COLUMN fetched SET DEFAULT (now() at time zone 'utc');
