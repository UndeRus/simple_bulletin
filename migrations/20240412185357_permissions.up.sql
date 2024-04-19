-- Add up migration script here

-- Create `groups` table.
create table if not exists groups (
    id integer primary key autoincrement,
    name text not null unique
);

-- Create `permissions` table.
create table if not exists permissions (
    id integer primary key autoincrement,
    name text not null unique
);


-- # Join tables.

-- Create `users_groups` table for many-to-many relationships between users and groups.
create table if not exists users_groups (
    user_id integer references users(id),
    group_id integer references groups(id),
    primary key (user_id, group_id)
);

-- Create `groups_permissions` table for many-to-many relationships between groups and permissions.
create table if not exists groups_permissions (
    group_id integer references groups(id),
    permission_id integer references permissions(id),
    primary key (group_id, permission_id)
);

insert into groups (name) values ('users');
insert into groups (name) values ('admins');

insert into permissions (name) values ('admin.read');
insert into permissions (name) values ('admin.write');
insert into permissions (name) values ('user.read');
insert into permissions (name) values ('user.write');

-- Insert group permissions.
insert into groups_permissions (group_id, permission_id)
values (
    (select id from groups where name = 'users'),
    (select id from permissions where name = 'user.read')
), (
    (select id from groups where name = 'users'),
    (select id from permissions where name = 'user.write')
), (
    (select id from groups where name = 'superusers'),
    (select id from permissions where name = 'admin.read')
), (
    (select id from groups where name = 'superusers'),
    (select id from permissions where name = 'admin.write')
);