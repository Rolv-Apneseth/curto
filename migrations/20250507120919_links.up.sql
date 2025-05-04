create table if not exists links
(
    id text not null primary key,
    target_url text not null,
    count_redirects integer default 0 not null,
    created_at timestamp default current_timestamp not null,
    updated_at timestamp default current_timestamp not null
);

-- Update `updated_at` automatically
CREATE OR REPLACE FUNCTION update_updated_at () RETURNS TRIGGER AS $$ BEGIN NEW.updated_at = now(); RETURN NEW; END; $$ language 'plpgsql' ;
CREATE TRIGGER update_updated_at_trigger BEFORE UPDATE ON links FOR EACH ROW EXECUTE PROCEDURE update_updated_at () ;
