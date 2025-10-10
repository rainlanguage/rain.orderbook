import type { RaindexClient, LocalDb } from '@rainlanguage/orderbook';
import type { SQLiteWasmDatabase } from '@rainlanguage/sqlite-web';
import { dbSyncIsActive, dbSyncIsRunning, dbSyncStatus } from '$lib/stores/dbSync';
import { get } from 'svelte/store';

interface StartLocalDbSyncOptions {
	raindexClient: RaindexClient;
	localDb: SQLiteWasmDatabase;
	chainId?: number;
	intervalMs?: number;
}

export function startLocalDbSync(options: StartLocalDbSyncOptions): () => void {
	const { raindexClient, localDb, chainId = 42161, intervalMs = 10_000 } = options;

	const queryFn = localDb.query.bind(localDb);
	// Ensure the Raindex client uses the WASM SQLite DB for its local queries
	raindexClient.setDbCallback(queryFn);

	const localDbClientResult = raindexClient.getLocalDbClient(chainId);
	if (localDbClientResult.error || !localDbClientResult.value) {
		const msg = localDbClientResult.error?.readableMsg ?? 'Failed to get local DB client';
		dbSyncStatus.set(msg);
		return () => {
			dbSyncIsActive.set(false);
			dbSyncIsRunning.set(false);
		};
	}

	let stopped = false;
	let isSyncing = false;
	let intervalId: ReturnType<typeof setInterval> | null = null;
	dbSyncIsActive.set(true);

	async function performSync() {
		if (isSyncing || stopped) return;

		isSyncing = true;
		dbSyncIsRunning.set(true);

		try {
			const syncResult = await raindexClient.syncLocalDatabase(
				queryFn,
				(status: string) => {
					dbSyncStatus.set(status);
				},
				chainId
			);

			if (syncResult.error) {
				dbSyncStatus.set(syncResult.error.readableMsg ?? syncResult.error.msg ?? 'Sync failed');
				return;
			}
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Sync failed';
			dbSyncStatus.set(message || 'Sync failed');
		} finally {
			dbSyncIsRunning.set(false);
			isSyncing = false;
		}
	}

	async function bootstrap() {
		dbSyncStatus.set('Starting database sync...');
		await performSync();

		if (!stopped) {
			intervalId = setInterval(() => {
				void performSync();
			}, intervalMs);
		}
	}

	void bootstrap();

	return () => {
		stopped = true;
		if (intervalId) {
			clearInterval(intervalId);
		}
		dbSyncIsActive.set(false);
		dbSyncIsRunning.set(false);
	};
}

if (import.meta.vitest) {
	const { describe, it, expect, beforeEach, afterEach, vi } = import.meta.vitest;
	type MockFn = ReturnType<typeof vi.fn>;
	interface MockedStartOptions {
		setDbCallback: MockFn;
		getLocalDbClient: MockFn;
		syncLocalDatabase: MockFn;
		query: MockFn;
	}

	const flushAsync = async () => {
		await Promise.resolve();
		await Promise.resolve();
	};

	const waitForSyncToSettle = async () => {
		for (let i = 0; i < 5 && get(dbSyncIsRunning); i++) {
			await flushAsync();
		}
	};

	const resetStores = () => {
		dbSyncStatus.set('');
		dbSyncIsActive.set(false);
		dbSyncIsRunning.set(false);
	};

	const createMocks = (): {
		raindexClient: RaindexClient;
		localDb: SQLiteWasmDatabase;
		localDbClient: LocalDb;
		deps: MockedStartOptions;
	} => {
		const setDbCallback = vi.fn();
		const getLocalDbClient = vi.fn();
		const syncLocalDatabase = vi.fn();
		const query = vi.fn();

		const raindexClient = {
			setDbCallback,
			getLocalDbClient,
			syncLocalDatabase
		} as unknown as RaindexClient;

		const localDb = {
			query
		} as unknown as SQLiteWasmDatabase;

		const localDbClient = {} as unknown as LocalDb;

		return {
			raindexClient,
			localDb,
			localDbClient,
			deps: { setDbCallback, getLocalDbClient, syncLocalDatabase, query }
		};
	};

	describe('startLocalDbSync', () => {
		beforeEach(() => {
			vi.useFakeTimers();
			resetStores();
		});

		afterEach(() => {
			vi.clearAllTimers();
			vi.useRealTimers();
		});

		it('returns cleanup when local DB client cannot be created', async () => {
			const { raindexClient, localDb, deps } = createMocks();

			const failureMessage = 'Failed to create client';
			deps.getLocalDbClient.mockReturnValue({
				error: { readableMsg: failureMessage }
			});

			const stop = startLocalDbSync({ raindexClient, localDb, chainId: 999 });

			expect(deps.setDbCallback).toHaveBeenCalledTimes(1);
			expect(typeof deps.setDbCallback.mock.calls[0][0]).toBe('function');
			expect(get(dbSyncStatus)).toBe(failureMessage);
			expect(deps.syncLocalDatabase).not.toHaveBeenCalled();
			expect(get(dbSyncIsActive)).toBe(false);
			expect(get(dbSyncIsRunning)).toBe(false);

			dbSyncIsActive.set(true);
			dbSyncIsRunning.set(true);
			stop();

			expect(get(dbSyncIsActive)).toBe(false);
			expect(get(dbSyncIsRunning)).toBe(false);
		});

		it('performs bootstrap sync and schedules periodic syncing', async () => {
			const { raindexClient, localDb, localDbClient, deps } = createMocks();

			const statusUpdates: string[] = [];

			deps.getLocalDbClient.mockReturnValue({ value: localDbClient, error: null });
			deps.syncLocalDatabase.mockImplementation(
				async (queryFn, statusCallback, incomingChainId) => {
					statusUpdates.push('Syncing...');
					statusCallback('Syncing...');
					expect(incomingChainId).toBe(42161);
					expect(queryFn).toBeTypeOf('function');
					return { error: null, value: true };
				}
			);

			const stop = startLocalDbSync({ raindexClient, localDb });

			await flushAsync();
			await waitForSyncToSettle();

			expect(get(dbSyncIsActive)).toBe(true);
			expect(get(dbSyncIsRunning)).toBe(false);
			expect(statusUpdates).toEqual(['Syncing...']);
			expect(get(dbSyncStatus)).toBe('Syncing...');
			expect(deps.syncLocalDatabase).toHaveBeenCalledTimes(1);

			expect(vi.getTimerCount()).toBeGreaterThan(0);
			await vi.advanceTimersByTimeAsync(10_000);
			await flushAsync();
			await waitForSyncToSettle();

			expect(deps.syncLocalDatabase).toHaveBeenCalledTimes(2);

			stop();
			expect(get(dbSyncIsActive)).toBe(false);
			expect(get(dbSyncIsRunning)).toBe(false);
			expect(vi.getTimerCount()).toBe(0);
		});

		it('surfaces sync errors and resets running state', async () => {
			const { raindexClient, localDb, localDbClient, deps } = createMocks();

			deps.getLocalDbClient.mockReturnValue({ value: localDbClient, error: null });
			deps.syncLocalDatabase.mockResolvedValue({
				error: { readableMsg: 'Sync failure', msg: 'raw failure' }
			});

			const stop = startLocalDbSync({ raindexClient, localDb, intervalMs: 5000 });

			await flushAsync();
			await waitForSyncToSettle();

			expect(get(dbSyncStatus)).toBe('Sync failure');
			expect(get(dbSyncIsRunning)).toBe(false);
			expect(deps.syncLocalDatabase).toHaveBeenCalledTimes(1);

			stop();
		});
	});
}
