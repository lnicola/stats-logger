create table tabs(time timestamptz not null primary key, tabs smallint not null);

select create_hypertable('tabs', 'time');
select set_chunk_time_interval('tabs', interval '1 month');
alter table tabs set(timescaledb.compress);
