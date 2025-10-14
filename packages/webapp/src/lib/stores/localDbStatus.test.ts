import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';

async function loadStoreModule() {
	return await import('./localDbStatus');
}

describe('localDbStatus store', () => {
	beforeEach(() => {
		vi.resetModules();
	});

	it('returns idle indicator when sync is disabled', async () => {
		const { localDbStatusIndicator } = await loadStoreModule();
		expect(get(localDbStatusIndicator)).toMatchObject({
			variant: 'idle',
			label: 'Sync paused'
		});
	});

	it('records latest status message and classifies success', async () => {
		const {
			localDbLatestEntry,
			localDbStatusIndicator,
			recordLocalDbStatus,
			setLocalDbSyncEnabled
		} = await loadStoreModule();

		setLocalDbSyncEnabled(true);
		recordLocalDbStatus('Database sync complete.');

		const latest = get(localDbLatestEntry);
		expect(latest?.message).toBe('Database sync complete.');
		expect(latest?.level).toBe('success');

		expect(get(localDbStatusIndicator)).toMatchObject({
			variant: 'success',
			label: 'Database sync complete.'
		});
	});

	it('classifies error messages appropriately', async () => {
		const { localDbStatusIndicator, recordLocalDbStatus, setLocalDbSyncEnabled } =
			await loadStoreModule();

		setLocalDbSyncEnabled(true);
		recordLocalDbStatus('Error syncing database');

		expect(get(localDbStatusIndicator)).toMatchObject({
			variant: 'error',
			label: 'Error syncing database'
		});
	});

	it('records explicit errors via recordLocalDbError', async () => {
		const { localDbStatusIndicator, recordLocalDbError, setLocalDbSyncEnabled } =
			await loadStoreModule();

		setLocalDbSyncEnabled(true);
		recordLocalDbError('Rate limited');

		expect(get(localDbStatusIndicator)).toMatchObject({
			variant: 'error',
			label: 'Rate limited'
		});
	});

	it('allows overriding status level explicitly', async () => {
		const { localDbStatusIndicator, recordLocalDbStatus, setLocalDbSyncEnabled } =
			await loadStoreModule();

		setLocalDbSyncEnabled(true);
		recordLocalDbStatus('Unexpected condition occurred', 'error');

		expect(get(localDbStatusIndicator)).toMatchObject({
			variant: 'error',
			label: 'Unexpected condition occurred'
		});
	});
});
