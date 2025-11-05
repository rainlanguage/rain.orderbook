import { getSyncStatus } from '@rainlanguage/orderbook';
import type { Hex, RaindexClient, SyncStatusResponse } from '@rainlanguage/orderbook';
import type { SQLiteWasmDatabase } from '@rainlanguage/sqlite-web';
import {
	dbSyncIsActive,
	dbSyncIsRunning,
	dbSyncLastBlock,
	dbSyncLastSyncTime,
	dbSyncStatus
} from '$lib/stores/dbSync';
import { get } from 'svelte/store';

interface StartLocalDbSyncOptions {
	raindexClient: RaindexClient;
	localDb: SQLiteWasmDatabase;
	chainId?: number;
	orderbookAddress?: Hex;
	intervalMs?: number;
}

// NOTE: This function will be replaced with a more robust solution
// after we introduce the new sync logic in the Raindex client.
export function startLocalDbSync(options: StartLocalDbSyncOptions): () => void {
	const {
		raindexClient,
		localDb,
		chainId = 42161,
		orderbookAddress = '0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB',
		intervalMs = 10_000
	} = options;

	const queryFn = localDb.query.bind(localDb);
	// Ensure the Raindex client uses the WASM SQLite DB for its local queries
	raindexClient.setDbCallback(queryFn);

	let stopped = false;
	let isSyncing = false;
	let intervalId: ReturnType<typeof setInterval> | null = null;

	dbSyncIsActive.set(true);

	async function updateSyncStatus() {
		try {
			const statusResult = await getSyncStatus(queryFn, chainId, orderbookAddress);
			if (!statusResult.error && statusResult.value && statusResult.value.length > 0) {
				const latestStatus = statusResult.value[statusResult.value.length - 1];
				dbSyncLastBlock.set(latestStatus.last_synced_block?.toString?.() ?? null);
				const syncTime = latestStatus.updated_at ? new Date(latestStatus.updated_at) : new Date();
				dbSyncLastSyncTime.set(syncTime);
			} else if (statusResult.error) {
				dbSyncStatus.set(
					statusResult.error.readableMsg ?? statusResult.error.msg ?? 'Failed to fetch sync status'
				);
			}
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Failed to update sync status';
			dbSyncStatus.set(message || 'Failed to update sync status');
		}
	}

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

			await updateSyncStatus();
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
		await updateSyncStatus();
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

	vi.mock('@rainlanguage/orderbook', async (importOriginal) => {
		return {
			...(await importOriginal()),
			getSyncStatus: vi.fn()
		};
	});

	const mockedGetSyncStatus = vi.mocked(getSyncStatus);
	type MockFn = ReturnType<typeof vi.fn>;
	interface MockedStartOptions {
		setDbCallback: MockFn;
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
		dbSyncLastBlock.set(null);
		dbSyncLastSyncTime.set(null);
		dbSyncIsActive.set(false);
		dbSyncIsRunning.set(false);
	};

	const createMocks = (): {
		raindexClient: RaindexClient;
		localDb: SQLiteWasmDatabase;
		deps: MockedStartOptions;
	} => {
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

		return {
			raindexClient,
			localDb,
			deps: { setDbCallback, syncLocalDatabase, query }
		};
	};

	describe('startLocalDbSync', () => {
		beforeEach(() => {
			vi.useFakeTimers();
			mockedGetSyncStatus.mockReset();
			mockedGetSyncStatus.mockResolvedValue({ error: undefined, value: [] });
			resetStores();
		});

		afterEach(() => {
			vi.clearAllTimers();
			vi.useRealTimers();
		});

		it('initializes sync and returns cleanup function', async () => {
			const { raindexClient, localDb, deps } = createMocks();

			deps.syncLocalDatabase.mockResolvedValue({
				error: undefined,
				value: true
			});

			const stop = startLocalDbSync({ raindexClient, localDb, chainId: 999 });

			expect(deps.setDbCallback).toHaveBeenCalledTimes(1);
			expect(typeof deps.setDbCallback.mock.calls[0][0]).toBe('function');
			expect(get(dbSyncStatus)).toBe('Starting database sync...');
			expect(get(dbSyncIsActive)).toBe(true);

			await flushAsync();
			await waitForSyncToSettle();

			expect(deps.syncLocalDatabase).toHaveBeenCalledTimes(1);
			expect(get(dbSyncIsRunning)).toBe(false);

			const boundQueryFn = deps.setDbCallback.mock.calls[0][0];
			const firstCall = mockedGetSyncStatus.mock.calls[0];
			expect(firstCall[0]).toBe(boundQueryFn);
			expect(firstCall[1]).toBe(999);
			expect(String(firstCall[2]).toLowerCase()).toBe('0x2f209e5b67a33b8fe96e28f24628df6da301c8eb');

			dbSyncIsActive.set(true);
			dbSyncIsRunning.set(true);
			stop();

			expect(get(dbSyncIsActive)).toBe(false);
			expect(get(dbSyncIsRunning)).toBe(false);
		});

		it('performs bootstrap sync and schedules periodic syncing', async () => {
			const { raindexClient, localDb, deps } = createMocks();

			const updatedAt = '2024-01-01T00:00:00.000Z';
			const statusUpdates: string[] = [];

			const syncStatus: SyncStatusResponse = {
				chain_id: 42161,
				orderbook_address: '0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB',
				last_synced_block: 123456,
				updated_at: updatedAt
			};

			mockedGetSyncStatus.mockResolvedValue({
				error: undefined,
				value: [syncStatus]
			});
			deps.syncLocalDatabase.mockImplementation(
				async (queryFn, statusCallback, incomingChainId) => {
					statusUpdates.push('Syncing...');
					statusCallback('Syncing...');
					expect(incomingChainId).toBe(42161);
					expect(queryFn).toBeTypeOf('function');
					return { error: undefined, value: true };
				}
			);

			const stop = startLocalDbSync({ raindexClient, localDb });

			await flushAsync();
			await waitForSyncToSettle();

			expect(get(dbSyncIsActive)).toBe(true);
			expect(get(dbSyncIsRunning)).toBe(false);
			expect(statusUpdates).toEqual(['Syncing...']);
			expect(get(dbSyncStatus)).toBe('Syncing...');
			expect(get(dbSyncLastBlock)).toBe('123456');
			expect(get(dbSyncLastSyncTime)?.toISOString()).toBe(updatedAt);
			expect(deps.syncLocalDatabase).toHaveBeenCalledTimes(1);
			expect(mockedGetSyncStatus).toHaveBeenCalledTimes(2);

			const boundQueryFn = deps.setDbCallback.mock.calls[0][0];
			const firstBootstrapCall = mockedGetSyncStatus.mock.calls[0];
			const secondBootstrapCall = mockedGetSyncStatus.mock.calls[1];
			const expectedAddress = '0x2f209e5b67a33b8fe96e28f24628df6da301c8eb';
			expect(firstBootstrapCall[0]).toBe(boundQueryFn);
			expect(firstBootstrapCall[1]).toBe(42161);
			expect(String(firstBootstrapCall[2]).toLowerCase()).toBe(expectedAddress);
			expect(secondBootstrapCall[0]).toBe(boundQueryFn);
			expect(secondBootstrapCall[1]).toBe(42161);
			expect(String(secondBootstrapCall[2]).toLowerCase()).toBe(expectedAddress);

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
			const { raindexClient, localDb, deps } = createMocks();

			deps.syncLocalDatabase.mockResolvedValue({
				error: { readableMsg: 'Sync failure', msg: 'raw failure' },
				value: undefined
			});

			const stop = startLocalDbSync({ raindexClient, localDb, intervalMs: 5000 });

			await flushAsync();
			await waitForSyncToSettle();

			expect(get(dbSyncStatus)).toBe('Sync failure');
			expect(get(dbSyncIsRunning)).toBe(false);
			expect(get(dbSyncLastBlock)).toBeNull();
			expect(mockedGetSyncStatus).toHaveBeenCalledTimes(1);
			expect(deps.syncLocalDatabase).toHaveBeenCalledTimes(1);

			stop();
		});
	});
}
