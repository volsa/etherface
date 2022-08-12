-- This file should undo anything in `up.sql`
DROP MATERIALIZED VIEW view_signature_insert_rate;
DROP MATERIALIZED VIEW view_signatures_popular_on_github;
DROP MATERIALIZED VIEW view_signature_kind_distribution;
DROP MATERIALIZED VIEW view_signature_count_statistics;
DROP TRIGGER trigger_refresh_materialized_views ON github_crawler_metadata;
DROP FUNCTION function_refresh_materialized_views;