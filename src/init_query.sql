CREATE TABLE IF NOT EXISTS files (
    key        TEXT    NOT NULL,
    time_stamp INTEGER NOT NULL,
    data       BLOB,
PRIMARY KEY (key, time_stamp));
