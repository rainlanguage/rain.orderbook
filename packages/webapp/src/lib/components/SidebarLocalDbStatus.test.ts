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
	const view = render(SidebarLocalDbStatus);

	const messages = view.getAllByText('Database sync complete.');
	expect(messages).toHaveLength(2);
	messages.forEach((message) => {
		expect(message).toBeVisible();
	});
	expect(screen.getByText('Active')).toBeInTheDocument();
	});

	it('uses error badge when the latest message contains an error', async () => {
	const store = await loadStoreModule();
	store.setLocalDbSyncEnabled(true);
	store.recordLocalDbError('Error syncing database');

	const { default: SidebarLocalDbStatus } = await import('./SidebarLocalDbStatus.svelte');
	const view = render(SidebarLocalDbStatus);

	const messages = view.getAllByText('Error syncing database');
	expect(messages).toHaveLength(2);
	messages.forEach((message) => {
		expect(message).toBeVisible();
	});
	expect(screen.getByText('Error')).toBeInTheDocument();
	});
});
