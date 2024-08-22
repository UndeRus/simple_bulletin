-- Add up migration script here
ALTER TABLE users ADD COLUMN active boolean DEFAULT false;