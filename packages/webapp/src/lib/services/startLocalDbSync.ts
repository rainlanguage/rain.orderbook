import type { RaindexClient, LocalDb } from '@rainlanguage/orderbook';
import type { SQLiteWasmDatabase } from 'sqlite-web';
import {
	dbSyncIsActive,
	dbSyncIsRunning,
	dbSyncLastBlock,
	dbSyncLastSyncTime,
	dbSyncStatus
} from '$lib/stores/dbSync';

interface StartLocalDbSyncOptions {
	raindexClient: RaindexClient;
	localDb: SQLiteWasmDatabase;
	chainId?: number;
	intervalMs?: number;
}

export function startLocalDbSync(options: StartLocalDbSyncOptions): () => void {
	const {
		raindexClient,
		localDb,
		chainId = 42161,
		intervalMs = 10_000
	} = options;

	const queryFn = localDb.query.bind(localDb);
	// Ensure the Raindex client uses the WASM SQLite DB for its local queries
	raindexClient.setDbCallback(queryFn);

	const localDbClientResult = raindexClient.getLocalDbClient(chainId);
	if (localDbClientResult.error || !localDbClientResult.value) {
		const msg = localDbClientResult.error?.readableMsg ?? 'Failed to get local DB client';
		dbSyncStatus.set(msg);
		console.error('startLocalDbSync: unable to create local DB client', localDbClientResult.error);
		return () => {
			dbSyncIsActive.set(false);
			dbSyncIsRunning.set(false);
		};
	}

	const localDbClient: LocalDb = localDbClientResult.value;

	let stopped = false;
	let isSyncing = false;
	let intervalId: ReturnType<typeof setInterval> | null = null;

	dbSyncIsActive.set(true);

	async function updateSyncStatus() {
		try {
			const statusResult = await localDbClient.getSyncStatus(queryFn);
			if (!statusResult.error && statusResult.value && statusResult.value.length > 0) {
				const latestStatus = statusResult.value[statusResult.value.length - 1];
				dbSyncLastBlock.set(latestStatus.last_synced_block?.toString?.() ?? null);
				const syncTime = latestStatus.updated_at ? new Date(latestStatus.updated_at) : new Date();
				dbSyncLastSyncTime.set(syncTime);
			} else if (statusResult.error) {
				console.warn('startLocalDbSync: getSyncStatus error', statusResult.error.readableMsg);
			}
		} catch (error) {
			console.error('startLocalDbSync: failed to update sync status', error);
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
				console.error('startLocalDbSync: sync error', syncResult.error);
				return;
			}

			await updateSyncStatus();
		} catch (error) {
			console.error('startLocalDbSync: sync threw', error);
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
