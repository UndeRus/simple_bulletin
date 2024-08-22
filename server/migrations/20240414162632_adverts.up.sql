-- Add up migration script here
CREATE TABLE if not exists adverts (
	id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	title TEXT NOT NULL,
	content TEXT NOT NULL,
    published BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE if not exists users_adverts (
    user_id INTEGER NOT NULL,
    advert_id INTEGER NOT NULL,
    primary key (user_id, advert_id)
);