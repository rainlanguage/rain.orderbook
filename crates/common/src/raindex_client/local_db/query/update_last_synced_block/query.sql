UPDATE sync_status SET last_synced_block = ?block_number, updated_at = CURRENT_TIMESTAMP WHERE id = 1;
