import type { RaindexClient } from '@rainlanguage/orderbook';
import type { SQLiteWasmDatabase } from '@rainlanguage/sqlite-web';

interface StartLocalDbSyncOptions {
	raindexClient: RaindexClient;
	localDb: SQLiteWasmDatabase;
	onStatusUpdate?: (status: string) => void;
}

export function startLocalDbSync({
	raindexClient,
	localDb,
	onStatusUpdate
}: StartLocalDbSyncOptions): void {
	const queryFn = localDb.query.bind(localDb);
	void raindexClient.setDbCallback(queryFn);

	const statusHandler =
		onStatusUpdate ??
		((status: string) => {
			console.log('[local-db-sync]', status);
		});

	void raindexClient.syncLocalDatabase(statusHandler);
}

if (import.meta.vitest) {
	const { describe, it, expect, vi } = import.meta.vitest;

	describe('startLocalDbSync', () => {
		it('sets the database callback and starts the sync loop', () => {
			const consoleSpy = vi.spyOn(console, 'debug').mockImplementation(() => undefined);
			const setDbCallback = vi.fn();
			const syncLocalDatabase = vi.fn();
			const query = vi.fn();

			const raindexClient = {
				setDbCallback,
				syncLocalDatabase
			} as unknown as RaindexClient;

			const localDb = {
				query
			} as unknown as SQLiteWasmDatabase;

			startLocalDbSync({ raindexClient, localDb });

			expect(setDbCallback).toHaveBeenCalledTimes(1);
			expect(typeof setDbCallback.mock.calls[0][0]).toBe('function');
			expect(syncLocalDatabase).toHaveBeenCalledTimes(1);

			const statusFn = syncLocalDatabase.mock.calls[0][0] as (status: string) => void;
			statusFn('Syncing...');
			expect(consoleSpy).toHaveBeenCalledWith('[local-db-sync]', 'Syncing...');

			consoleSpy.mockRestore();
		});

		it('uses provided status handler when supplied', () => {
			const setDbCallback = vi.fn();
			const syncLocalDatabase = vi.fn();
			const query = vi.fn();
			const onStatusUpdate = vi.fn();

			const raindexClient = {
				setDbCallback,
				syncLocalDatabase
			} as unknown as RaindexClient;

			const localDb = {
				query
			} as unknown as SQLiteWasmDatabase;

			startLocalDbSync({ raindexClient, localDb, onStatusUpdate });

			const statusFn = syncLocalDatabase.mock.calls[0][0] as (status: string) => void;
			statusFn('Done');

			expect(onStatusUpdate).toHaveBeenCalledWith('Done');
		});
	});
}
