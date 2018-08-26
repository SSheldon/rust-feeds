CREATE TABLE item (
  id SERIAL PRIMARY KEY,
  url VARCHAR NOT NULL,
  title VARCHAR NOT NULL,
  content TEXT NOT NULL,
  published TIMESTAMP NOT NULL DEFAULT (now() at time zone 'utc')
)
