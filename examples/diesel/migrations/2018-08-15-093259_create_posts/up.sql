create table posts (
    id serial primary key,
    title varchar not null,
    body text not null,
    published boolean not null default false
)
