BEGIN TRANSACTION;

DROP TABLE IF EXISTS context_values;
DROP TABLE IF EXISTS take_order_contexts;
DROP TABLE IF EXISTS take_orders;
DROP TABLE IF EXISTS order_ios;
DROP TABLE IF EXISTS order_events;
DROP TABLE IF EXISTS withdrawals;
DROP TABLE IF EXISTS deposits;
DROP TABLE IF EXISTS clear_v3_events;
DROP TABLE IF EXISTS after_clear_v2_events;
DROP TABLE IF EXISTS meta_events;
DROP TABLE IF EXISTS erc20_tokens;
DROP TABLE IF EXISTS interpreter_store_sets;
DROP TABLE IF EXISTS sync_status;

COMMIT;

VACUUM;
