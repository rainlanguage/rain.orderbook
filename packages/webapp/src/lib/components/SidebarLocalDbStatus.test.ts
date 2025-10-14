import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';

async function loadStoreModule() {
	return await import('../stores/localDbStatus');
}

describe('SidebarLocalDbStatus', () => {
	beforeEach(() => {
		vi.resetModules();
	});

	it('shows placeholder when no status has been recorded', async () => {
		const { default: SidebarLocalDbStatus } = await import('./SidebarLocalDbStatus.svelte');

		render(SidebarLocalDbStatus);

		expect(
			screen.getByText('Status updates will appear here once the local indexer runs.')
		).toBeInTheDocument();
		expect(screen.getByText('Paused')).toBeInTheDocument();
	});

	it('renders the latest SDK status message', async () => {
		const store = await loadStoreModule();
		store.setLocalDbSyncEnabled(true);

		vi.setSystemTime(new Date('2024-02-01T12:34:56Z'));

		store.recordLocalDbStatus('Database sync complete.');

		const { default: SidebarLocalDbStatus } = await import('./SidebarLocalDbStatus.svelte');
		render(SidebarLocalDbStatus);

		const entries = screen.getAllByText('Database sync complete.');
		expect(entries).toHaveLength(1);
		expect(entries[0]).toBeVisible();
		expect(entries[0]).toHaveAttribute('title', 'Database sync complete.');
		const badge = screen.getByText('Active');
		expect(badge).toBeVisible();
	});

	it('uses error badge when the latest message contains an error', async () => {
		const store = await loadStoreModule();
		store.setLocalDbSyncEnabled(true);
		store.recordLocalDbError('Error syncing database');

		const { default: SidebarLocalDbStatus } = await import('./SidebarLocalDbStatus.svelte');
		render(SidebarLocalDbStatus);

		const messages = screen.getAllByText('Error syncing database');
		expect(messages).toHaveLength(1);
		expect(messages[0]).toBeVisible();
		expect(messages[0]).toHaveAttribute('title', 'Error syncing database');
		const badge = screen.getByText('Error');
		expect(badge).toBeVisible();
	});
});
