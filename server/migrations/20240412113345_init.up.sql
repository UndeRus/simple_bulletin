-- Add up migration script here
-- Create users table.
create table if not exists users
(
    id           integer primary key autoincrement,
    username     text not null unique,
    password_hash text not null
);