-- This file should undo anything in `up.sql`
DROP TABLE mapping_signature_etherscan;
DROP TABLE mapping_signature_fourbyte;
DROP TABLE mapping_signature_github;
DROP TABLE mapping_stargazer;

DROP TABLE signature;
DROP TABLE etherscan_contract;
DROP TABLE github_repository;
DROP TABLE github_user;
DROP TABLE github_crawler_metadata;

DROP TYPE signature_kind;
DROP TYPE signature_visibility;