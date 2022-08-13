CREATE EXTENSION pg_trgm;
CREATE INDEX index_trgm_ops__signature_text ON signature USING gin (text gin_trgm_ops);
CREATE INDEX index_trgm_ops__signature_hash ON signature USING gin (hash gin_trgm_ops); 
VACUUM FULL ANALYZE;