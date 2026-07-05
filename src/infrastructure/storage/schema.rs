// @generated automatically by Diesel CLI.

diesel::table! {
    citations (id) {
        id -> Integer,
        report_id -> Integer,
        reference_number -> Integer,
        url -> Text,
        title -> Text,
        context_snippet -> Text,
        verification_level -> Text,
    }
}

diesel::table! {
    log_entries (id) {
        id -> Integer,
        session_id -> Text,
        agent_tag -> Text,
        message -> Text,
        level -> Text,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    research_reports (id) {
        id -> Integer,
        session_id -> Text,
        title -> Text,
        content -> Text,
        stats_json -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    sessions (id) {
        id -> Text,
        query -> Text,
        status -> Text,
        pipeline_stage -> Text,
        plan_json -> Nullable<Text>,
        clarifier_result -> Nullable<Text>,
        telemetry_json -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    sources (id) {
        id -> Integer,
        session_id -> Text,
        url -> Text,
        title -> Text,
        snippet -> Text,
        relevance -> Double,
        source_type -> Text,
        verification_status -> Text,
        embedding_id -> Nullable<Text>,
    }
}

diesel::joinable!(citations -> research_reports (report_id));
diesel::joinable!(log_entries -> sessions (session_id));
diesel::joinable!(research_reports -> sessions (session_id));
diesel::joinable!(sources -> sessions (session_id));

diesel::allow_tables_to_appear_in_same_query!(
    citations,
    log_entries,
    research_reports,
    sessions,
    sources,
);
