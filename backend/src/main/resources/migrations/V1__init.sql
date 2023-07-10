

create table stations (
    id UUID NOT NULL PRIMARY KEY,
    mac_addr BYTEA not null,
    name TEXT not null,
    description TEXT not null,
    location TEXT not null,
    watering_schedule JSON not null,
    user_id TEXT not null,
    created TIMESTAMPTZ not null,
    updated TIMESTAMPTZ
);

create table station_log (
    station_id UUID NOT NULL REFERENCES stations (id) ON DELETE CASCADE,
    occurred_on TIMESTAMPTZ NOT NULL,
    event JSON not null
);