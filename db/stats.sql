create table stats(time timestamptz not null primary key, temperature smallint not null, humidity smallint not null);
create table stats2(time timestamptz not null primary key, temperature smallint not null, co2 smallint not null);

create procedure insert_stats("time" timestamptz, temperature real, humidity real)
    language sql
    security definer
as $$
    insert into stats(time, temperature, humidity) values (time, (temperature * 100) :: smallint, (humidity * 100) :: smallint);
$$;
create procedure insert_stats2("time" timestamptz, temperature real, co2 smallint)
    language sql
    security definer
as $$
    insert into stats2(time, temperature, co2) values (time, (temperature * 100) :: smallint, co2);
$$;

create view v_stats as select time, temperature :: real / 100.0 :: real as temperature, humidity :: real / 100.0 :: real as humidity from stats;
create view v_stats2 as select time, temperature :: real / 100.0 :: real as temperature, co2 from stats2;

select create_hypertable('stats', 'time');
select create_hypertable('stats2', 'time');
select set_chunk_time_interval('stats', interval '1 month');
select set_chunk_time_interval('stats2', interval '1 month');
alter table stats set(timescaledb.compress);
alter table stats2 set(timescaledb.compress);
