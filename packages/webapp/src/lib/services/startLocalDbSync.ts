import type { RaindexClient, WasmEncodedResult } from '@rainlanguage/orderbook';
import type { SQLiteWasmDatabase } from '@rainlanguage/sqlite-web';
import {
	recordLocalDbError,
	recordLocalDbStatus,
	setLocalDbSyncEnabled
} from '$lib/stores/localDbStatus';

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

	setLocalDbSyncEnabled(true);

	let statusErrorHandled = false;

	const statusHandler = (result?: WasmEncodedResult<string | undefined>) => {
		if (!result) return;

		if (result.error) {
			const message =
				result.error.readableMsg && result.error.readableMsg.trim().length > 0
					? result.error.readableMsg
					: typeof result.error.msg === 'string'
						? result.error.msg
						: 'Local DB sync failed.';
			recordLocalDbError(message);
			statusErrorHandled = true;
			return;
		}

		const message = result.value;
		if (!message) return;

		recordLocalDbStatus(message);
		statusErrorHandled = false;

		if (onStatusUpdate) {
			onStatusUpdate(message);
			return;
		}
	};

	const syncResult = raindexClient.syncLocalDatabase(statusHandler);

	void Promise.resolve(syncResult)
		.then((result?: WasmEncodedResult<void>) => {
			const errorInfo = result?.error;
			if (!errorInfo || statusErrorHandled) return;

			const readableMessage =
				(errorInfo.readableMsg && errorInfo.readableMsg.trim().length > 0
					? errorInfo.readableMsg
					: typeof errorInfo.msg === 'string'
						? errorInfo.msg
						: null) ?? 'Local DB sync failed.';

			recordLocalDbError(readableMessage);
		})
		.catch((error) => {
			if (statusErrorHandled) return;
			const errorMessage =
				error instanceof Error
					? error.message
					: typeof error === 'string'
						? error
						: JSON.stringify(error);
			recordLocalDbError(`Local DB sync failed: ${errorMessage}`);
		});
}

if (import.meta.vitest) {
	const { describe, it, expect, vi, beforeEach } = import.meta.vitest;
	type Mock = ReturnType<typeof vi.fn>;

	vi.mock('$lib/stores/localDbStatus', () => {
		return {
			recordLocalDbStatus: vi.fn(),
			recordLocalDbError: vi.fn(),
			setLocalDbSyncEnabled: vi.fn()
		};
	});

	describe('startLocalDbSync', () => {
		const recordLocalDbStatusMock = recordLocalDbStatus as unknown as Mock;
		const recordLocalDbErrorMock = recordLocalDbError as unknown as Mock;
		const setLocalDbSyncEnabledMock = setLocalDbSyncEnabled as unknown as Mock;

		beforeEach(() => {
			vi.clearAllMocks();
		});

		it('sets the database callback and starts the sync loop', () => {
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
			expect(setLocalDbSyncEnabledMock).toHaveBeenCalledWith(true);

			const statusFn = syncLocalDatabase.mock.calls[0][0] as (
				status: WasmEncodedResult<string>
			) => void;
			statusFn({ value: 'Syncing...', error: undefined });
			expect(recordLocalDbStatusMock).toHaveBeenCalledWith('Syncing...');
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

			const statusFn = syncLocalDatabase.mock.calls[0][0] as (
				status: WasmEncodedResult<string>
			) => void;
			statusFn({ value: 'Done', error: undefined });

			expect(onStatusUpdate).toHaveBeenCalledWith('Done');
			expect(recordLocalDbStatusMock).toHaveBeenCalledWith('Done');
		});

		it('records an error when status payload includes error', () => {
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

			const statusFn = syncLocalDatabase.mock.calls[0][0] as (
				status: WasmEncodedResult<string>
			) => void;
			statusFn({
				value: undefined,
				error: {
					readableMsg: 'Rate limited',
					msg: '429'
				}
			});

			expect(recordLocalDbErrorMock).toHaveBeenCalledWith('Rate limited');
		});

		it('records an error when sync resolves with error payload', async () => {
			const setDbCallback = vi.fn();
			const syncLocalDatabase = vi.fn().mockReturnValue(
				Promise.resolve({
					value: undefined,
					error: { readableMsg: 'Boom' }
				})
			);
			const query = vi.fn();

			const raindexClient = {
				setDbCallback,
				syncLocalDatabase
			} as unknown as RaindexClient;

			const localDb = {
				query
			} as unknown as SQLiteWasmDatabase;

			startLocalDbSync({ raindexClient, localDb });

			const syncResult = syncLocalDatabase.mock.results[0]?.value;
			await syncResult;
			await Promise.resolve();

			expect(recordLocalDbErrorMock).toHaveBeenCalledWith('Boom');
		});

		it('records an error when sync rejects', async () => {
			const setDbCallback = vi.fn();
			const syncLocalDatabase = vi.fn().mockReturnValue(Promise.reject(new Error('offline')));
			const query = vi.fn();

			const raindexClient = {
				setDbCallback,
				syncLocalDatabase
			} as unknown as RaindexClient;

			const localDb = {
				query
			} as unknown as SQLiteWasmDatabase;

			startLocalDbSync({ raindexClient, localDb });

			try {
				const syncResult = syncLocalDatabase.mock.results[0]?.value;
				await syncResult;
			} catch {
				// already handled by catch branch in startLocalDbSync
			}
			await Promise.resolve();

			expect(recordLocalDbErrorMock).toHaveBeenCalledWith('Local DB sync failed: offline');
		});
	});
}
