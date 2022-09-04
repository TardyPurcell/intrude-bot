DROP TABLE IF EXISTS integral_time_card;
CREATE TABLE integral_time_card(
    user_id INTEGER PRIMARY KEY,
    started_at DATETIME NOT NULL ,
    updated_at DATETIME NOT NULL
);