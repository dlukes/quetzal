table! {
    corpora (id) {
        id -> Nullable<Integer>,
        label -> Text,
    }
}

table! {
    doc2speaker (id) {
        id -> Integer,
        doc_id -> Integer,
        speaker_id -> Integer,
        words -> Nullable<Integer>,
    }
}

table! {
    docs (id) {
        id -> Nullable<Integer>,
        project_id -> Integer,
        corpus_id -> Nullable<Integer>,
        assigned_to_id -> Nullable<Integer>,
        assigned_by_id -> Nullable<Integer>,
        done -> Nullable<Bool>,
        date -> Timestamp,
        place_id -> Integer,
    }
}

table! {
    enum_educations (id) {
        id -> Nullable<Integer>,
        label -> Text,
    }
}

table! {
    enum_genders (id) {
        id -> Nullable<Integer>,
        label -> Text,
    }
}

table! {
    enum_places (id) {
        id -> Nullable<Integer>,
        label -> Text,
        region_id -> Integer,
    }
}

table! {
    enum_regions (id) {
        id -> Nullable<Integer>,
        label -> Text,
    }
}

table! {
    enum_roles (id) {
        id -> Nullable<Integer>,
        label -> Text,
    }
}

table! {
    projects (id) {
        id -> Nullable<Integer>,
        label -> Text,
        badge -> Text,
    }
}

table! {
    speakers (id) {
        id -> Nullable<Integer>,
        user_id -> Integer,
        project_id -> Integer,
        nickname -> Text,
        gender_id -> Integer,
        education_id -> Integer,
        place_id -> Integer,
        year -> Integer,
    }
}

table! {
    users (id) {
        id -> Nullable<Integer>,
        username -> Text,
        role_id -> Integer,
        badge -> Nullable<Text>,
        supervisor_id -> Nullable<Integer>,
    }
}

joinable!(doc2speaker -> docs (doc_id));
joinable!(doc2speaker -> speakers (speaker_id));
joinable!(docs -> corpora (corpus_id));
joinable!(docs -> projects (project_id));
joinable!(enum_places -> enum_regions (region_id));
joinable!(speakers -> projects (project_id));
joinable!(speakers -> users (user_id));
joinable!(users -> enum_roles (role_id));

allow_tables_to_appear_in_same_query!(
    corpora,
    doc2speaker,
    docs,
    enum_educations,
    enum_genders,
    enum_places,
    enum_regions,
    enum_roles,
    projects,
    speakers,
    users,
);
