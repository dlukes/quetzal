-- Immutable tables (= "enums") {{{1

-- TODO: Add a "hidden" column to all enums with a default value of
-- false, which will allow elegant deprecation of enum variants by first
-- making it impossible to set these values for new records. Also,
-- document this recommended workflow somewhere.

-- Roles {{{2

create table enum_roles (
  id integer primary key not null,
  label text unique not null
);
-- regular users can access only their own data, supervisors can access
-- their own data + the data of their supervisees, admins can access
-- all data
insert into enum_roles (label) values ('regular'), ('supervisor'), ('admin');

-- Genders {{{2

create table enum_genders (
  id integer primary key not null,
  label text unique not null
);
insert into enum_genders (label) values ('muž'), ('žena');

-- Educations {{{2

create table enum_educations (
  id integer primary key not null,
  label text unique not null
);
insert into enum_educations (label) values ('ZŠ'), ('SŠ'), ('SOŠ'), ('VŠ');

-- Regions {{{2

create table enum_regions (
  id integer primary key not null,
  label text unique not null
);
insert into enum_regions (label) values
  ('severovýchodočeská'),
  ('středočeská'),
  ('západočeská'),
  ('jihočeská'),
  ('českomoravská'),
  ('středomoravská'),
  ('východomoravská'),
  ('slezská'),
  ('pohraničí české'),
  ('pohraničí moravské a slezské'),
  ('zahraničí');

-- Places {{{2

create table enum_places (
  id integer primary key not null,
  label text unique not null,
  region_id integer not null references enum_regions (id)
    on update cascade on delete restrict
);
insert into enum_places (label, region_id) values
  -- TODO: this of course needs to be the full exhaustive list that
  -- Hanka is building
  ('Praha', 2),
  ('Brno', 6),
  ('Ostrava', 8);

-- Views {{{2

create view view_geo as
  select
    enum_places.id as place_id,
    enum_places.label as place,
    enum_regions.label as region
  from enum_places
  join enum_regions on enum_places.region_id = enum_regions.id;

-- Mutable tables {{{1

-- Project management {{{2

create table users (
  id integer primary key not null,
  username text unique not null,
  -- password text not null,
  -- email text not null,
  -- name text not null,
  -- surname text not null,
  role_id integer not null references enum_roles (id)
    on update cascade on delete restrict,

  -- for generating document labels; only supervisors need this
  badge text unique,
  supervisor_id integer references users (id)
    on update cascade on delete restrict
);

-- projects to which speakers and documents belong, which may affect
-- which metadata is collected
create table projects (
  id integer primary key not null,
  label text unique not null,
  -- for generating document labels
  badge text unique not null
);

-- corpora to which documents can be assigned
create table corpora (
  id integer primary key not null,
  label text unique not null
);

-- Data {{{2

create table speakers (
  id integer primary key not null,
  user_id integer not null references users (id)
    on update cascade on delete restrict,
  project_id integer not null references projects (id)
    on update cascade on delete restrict,

  -- TODO: make sure nicknames are only visible to people who can edit
  -- the given speaker...?
  nickname text not null,
  gender_id integer not null,
  education_id integer not null,
  place_id integer not null,
  year integer not null
);

create table docs (
  id integer primary key not null,
  project_id integer not null references projects (id)
    on update cascade on delete restrict,
  corpus_id integer references corpora (id)
    on update cascade on delete restrict,

  -- the following columns keep track of whether anybody's been assigned
  -- to work on the document, who assigned them, and whether the work is
  -- done and the supervisor needs to check it
  assigned_to_id integer references users (id)
    on update cascade on delete restrict,
  assigned_by_id integer references users (id)
    on update cascade on delete restrict,
  done boolean,

  date timestamp not null,
  place_id integer not null
);

create table doc2speaker (
  id integer primary key not null,
  doc_id integer not null references docs (id)
    on update cascade on delete restrict,
  speaker_id integer not null references speakers (id)
    on update cascade on delete restrict,
  words integer
);

-- Views {{{2

create view view_speakers as
  select
    speakers.id as id,
    projects.label as project,
    nickname,
    enum_genders.label as gender,
    enum_educations.label as education,
    place,
    region,
    year
  from speakers
  join projects on projects.id = project_id
  join enum_genders on enum_genders.id = gender_id
  join enum_educations on enum_educations.id = education_id
  natural join view_geo;

create view view_docs as
  select
    docs.id as id,
    projects.label as project,
    corpora.label as corpus,
    place,
    region,
    date
  from docs
  join projects on projects.id = project_id
  join corpora on corpora.id = corpus_id
  natural join view_geo;

create view view_doc2speaker as
  select
    doc2speaker.id as id,
    view_docs.project as project,
    view_docs.corpus as corpus,
    view_docs.place as doc_place,
    view_docs.region as doc_region,
    gender,
    (case when date - year < 35 then 'mladší' else 'starší' end) as age,
    (case when education = 'VŠ' then 'vyšší' else 'nižší' end) as education,
    view_speakers.place as spk_place,
    view_speakers.region as spk_region,
    words
  from doc2speaker
  join view_speakers on doc2speaker.speaker_id = view_speakers.id
  join view_docs on doc2speaker.doc_id = view_docs.id;

-- Toy data for mutable tables {{{1

insert into users (username, role_id, badge, supervisor_id) values
  ('admin', 3, 'A', null),
  ('supervisor', 2, 'S', 1),
  ('regular', 1, null, 2);

insert into projects (label, badge) values
  ('neformální', 'N'),
  ('formální', 'F');

insert into corpora (label) values
  ('ortofon');

insert into speakers (user_id, project_id, nickname, gender_id,
  education_id, place_id, year) values
  (3, 1, 'John Doe', 1, 1, 1, 1988),
  (3, 1, 'Jane Doe', 2, 4, 2, 1984);

insert into docs (project_id, corpus_id, date, place_id) values
  (1, 1, '2019-03-01', 3);

insert into doc2speaker (doc_id, speaker_id, words) values
  (1, 1, 1000),
  (1, 2, 2000);

-- vim: foldmethod=marker:
