CREATE TABLE read (
  profile_id INTEGER REFERENCES profile,
  item_id INTEGER REFERENCES item,
  is_read BOOLEAN NOT NULL,
  is_saved BOOLEAN NOT NULL,
  PRIMARY KEY (profile_id, item_id)
);
