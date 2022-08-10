CREATE MATERIALIZED VIEW view_signature_insert_rate AS 
	SELECT DATE(date_trunc('day', added_at)) AS date, COUNT(*) AS count FROM signature WHERE added_at > (CURRENT_DATE - INTERVAL '31 days') GROUP BY 1;

CREATE MATERIALIZED VIEW view_signatures_popular_on_github AS 
	SELECT signature."text", COUNT(*) FROM signature JOIN mapping_signature_github ON signature.id = mapping_signature_github.signature_id GROUP BY 1 ORDER BY 2 DESC LIMIT 100;

CREATE MATERIALIZED VIEW view_signature_kind_distribution AS 
	SELECT kind, COUNT(*) FROM mapping_signature_kind GROUP BY 1;

CREATE MATERIALIZED VIEW view_signature_count_statistics AS 
	SELECT 	(SELECT COUNT(*) as signature_count FROM signature) 
				AS signature_count, 
			(SELECT COUNT(DISTINCT signature_id) AS signature_count_github FROM mapping_signature_github) 
				AS signature_count_github,
			(SELECT COUNT(DISTINCT signature_id) AS signature_count_etherscan FROM mapping_signature_etherscan)
				AS signature_count_etherscan,
			(SELECT COUNT(DISTINCT signature_id) AS signature_count_fourbyte FROM mapping_signature_fourbyte) 
				AS signature_count_fourbyte,
			(SELECT AVG(added_at_count)::BIGINT FROM (SELECT date_trunc('day', added_at), COUNT(*) AS added_at_count FROM signature WHERE added_at > CURRENT_DATE - 7 GROUP BY 1) AS temp)
				AS average_daily_signature_insert_rate_last_week,
			(SELECT AVG(added_at_count)::BIGINT FROM (SELECT date_trunc('day', added_at), COUNT(*) AS added_at_count FROM signature WHERE added_at < CURRENT_DATE - 7 AND added_at > CURRENT_DATE - 14 GROUP BY 1) AS temp)
				AS average_daily_signature_insert_rate_week_before_last;


-- Instead of using e.g. a cron job to periodically updated the defined views we'll use a trigger instead which
-- fires whenever the github_crawler_metadata.last_repository_search row is updated. This is ideal because
-- in theory the given row should updated every 24 hours, as such our views update every 24 hours.
CREATE OR REPLACE FUNCTION function_refresh_materialized_views() RETURNS TRIGGER AS $trigger_refresh_materialized_views$
BEGIN
	REFRESH MATERIALIZED VIEW view_signature_insert_rate;
	REFRESH MATERIALIZED VIEW view_signatures_popular_on_github;
	REFRESH MATERIALIZED VIEW view_signature_kind_distribution;
	REFRESH MATERIALIZED VIEW view_signature_count_statistics;
	RETURN NULL;
END $trigger_refresh_materialized_views$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER trigger_refresh_materialized_views
	AFTER UPDATE OF last_repository_search ON github_crawler_metadata 
	FOR EACH STATEMENT 
	EXECUTE FUNCTION function_refresh_materialized_views();