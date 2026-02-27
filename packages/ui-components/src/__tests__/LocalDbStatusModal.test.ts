import { describe, it, expect, afterEach, vi } from 'vitest';
import { render, screen, cleanup } from '@testing-library/svelte';
import LocalDbStatusModal from '../lib/components/LocalDbStatusModal.svelte';
import type { NetworkSyncStatus, OrderbookSyncStatus } from '@rainlanguage/orderbook';

vi.mock('$lib/utils/getNetworkName', () => ({
	getNetworkName: (chainId: number) => {
		const names: Record<number, string> = {
			137: 'Polygon',
			42161: 'Arbitrum',
			8453: 'Base'
		};
		return names[chainId] ?? null;
	}
}));

describe('LocalDbStatusModal', () => {
	afterEach(() => {
		cleanup();
	});

	it('renders modal with header when open', () => {
		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses: new Map(),
				orderbookStatuses: new Map()
			}
		});

		expect(screen.getByText('Database Sync Status')).toBeInTheDocument();
	});

	it('shows empty state message when no networks are syncing', () => {
		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses: new Map(),
				orderbookStatuses: new Map()
			}
		});

		expect(screen.getByText('No networks are being synced.')).toBeInTheDocument();
	});

	it('renders network groups with correct network names', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'active', schedulerState: 'leader' }],
			[42161, { chainId: 42161, status: 'syncing', schedulerState: 'leader' }]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				orderbookStatuses: new Map()
			}
		});

		expect(screen.getByText('Polygon')).toBeInTheDocument();
		expect(screen.getByText('Arbitrum')).toBeInTheDocument();
	});

	it('falls back to chain ID when network name is unknown', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[99999, { chainId: 99999, status: 'active', schedulerState: 'leader' }]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				orderbookStatuses: new Map()
			}
		});

		expect(screen.getByText('Chain 99999')).toBeInTheDocument();
	});

	it('shows Observing badge when schedulerState is notLeader', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'active', schedulerState: 'notLeader' }]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				orderbookStatuses: new Map()
			}
		});

		expect(screen.getByText('Observing')).toBeInTheDocument();
	});

	it('does not show Observing badge when schedulerState is leader', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'active', schedulerState: 'leader' }]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				orderbookStatuses: new Map()
			}
		});

		expect(screen.queryByText('Observing')).not.toBeInTheDocument();
	});

	it('displays orderbook addresses under their network', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'active', schedulerState: 'leader' }]
		]);
		const orderbookStatuses = new Map<string, OrderbookSyncStatus>([
			[
				'137:0x1234567890123456789012345678901234567890',
				{
					obId: {
						chainId: 137,
						orderbookAddress: '0x1234567890123456789012345678901234567890'
					},
					status: 'active',
					schedulerState: 'leader'
				}
			]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				orderbookStatuses
			}
		});

		expect(screen.getByText('0x1234567890123456789012345678901234567890')).toBeInTheDocument();
	});

	it('shows phase message when orderbook is syncing', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'syncing', schedulerState: 'leader' }]
		]);
		const orderbookStatuses = new Map<string, OrderbookSyncStatus>([
			[
				'137:0x1234567890123456789012345678901234567890',
				{
					obId: {
						chainId: 137,
						orderbookAddress: '0x1234567890123456789012345678901234567890'
					},
					status: 'syncing',
					schedulerState: 'leader',
					phaseMessage: 'Fetching latest block'
				}
			]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				orderbookStatuses
			}
		});

		expect(screen.getByText('Fetching latest block')).toBeInTheDocument();
	});

	it('does not show phase message when orderbook status is not syncing', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'active', schedulerState: 'leader' }]
		]);
		const orderbookStatuses = new Map<string, OrderbookSyncStatus>([
			[
				'137:0x1234567890123456789012345678901234567890',
				{
					obId: {
						chainId: 137,
						orderbookAddress: '0x1234567890123456789012345678901234567890'
					},
					status: 'active',
					schedulerState: 'leader',
					phaseMessage: 'This should not appear'
				}
			]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				orderbookStatuses
			}
		});

		expect(screen.queryByText('This should not appear')).not.toBeInTheDocument();
	});

	it('shows error message when orderbook has failure', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'failure', schedulerState: 'leader' }]
		]);
		const orderbookStatuses = new Map<string, OrderbookSyncStatus>([
			[
				'137:0x1234567890123456789012345678901234567890',
				{
					obId: {
						chainId: 137,
						orderbookAddress: '0x1234567890123456789012345678901234567890'
					},
					status: 'failure',
					schedulerState: 'leader',
					error: 'RPC connection failed'
				}
			]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				orderbookStatuses
			}
		});

		expect(screen.getByText('RPC connection failed')).toBeInTheDocument();
	});

	it('shows network-level error when network has failure', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[
				137,
				{
					chainId: 137,
					status: 'failure',
					schedulerState: 'leader',
					error: 'Network initialization failed'
				}
			]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				orderbookStatuses: new Map()
			}
		});

		expect(screen.getByText('Network initialization failed')).toBeInTheDocument();
	});

	it('groups orderbooks correctly by chain ID', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'active', schedulerState: 'leader' }],
			[42161, { chainId: 42161, status: 'active', schedulerState: 'leader' }]
		]);
		const orderbookStatuses = new Map<string, OrderbookSyncStatus>([
			[
				'137:0x1111111111111111111111111111111111111111',
				{
					obId: {
						chainId: 137,
						orderbookAddress: '0x1111111111111111111111111111111111111111'
					},
					status: 'active',
					schedulerState: 'leader'
				}
			],
			[
				'42161:0x2222222222222222222222222222222222222222',
				{
					obId: {
						chainId: 42161,
						orderbookAddress: '0x2222222222222222222222222222222222222222'
					},
					status: 'syncing',
					schedulerState: 'leader'
				}
			]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				orderbookStatuses
			}
		});

		const polygonGroup = screen.getByTestId('network-group-137');
		const arbitrumGroup = screen.getByTestId('network-group-42161');

		expect(polygonGroup).toBeInTheDocument();
		expect(arbitrumGroup).toBeInTheDocument();
		expect(screen.getByText('0x1111111111111111111111111111111111111111')).toBeInTheDocument();
		expect(screen.getByText('0x2222222222222222222222222222222222222222')).toBeInTheDocument();
	});

	it('handles multiple orderbooks on the same network', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'active', schedulerState: 'leader' }]
		]);
		const orderbookStatuses = new Map<string, OrderbookSyncStatus>([
			[
				'137:0x1111111111111111111111111111111111111111',
				{
					obId: {
						chainId: 137,
						orderbookAddress: '0x1111111111111111111111111111111111111111'
					},
					status: 'active',
					schedulerState: 'leader'
				}
			],
			[
				'137:0x2222222222222222222222222222222222222222',
				{
					obId: {
						chainId: 137,
						orderbookAddress: '0x2222222222222222222222222222222222222222'
					},
					status: 'syncing',
					schedulerState: 'leader',
					phaseMessage: 'Running bootstrap'
				}
			]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				orderbookStatuses
			}
		});

		expect(screen.getByText('0x1111111111111111111111111111111111111111')).toBeInTheDocument();
		expect(screen.getByText('0x2222222222222222222222222222222222222222')).toBeInTheDocument();
		expect(screen.getByText('Running bootstrap')).toBeInTheDocument();
	});

	it('does not show orderbook error when status is not failure', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'active', schedulerState: 'leader' }]
		]);
		const orderbookStatuses = new Map<string, OrderbookSyncStatus>([
			[
				'137:0x1234567890123456789012345678901234567890',
				{
					obId: {
						chainId: 137,
						orderbookAddress: '0x1234567890123456789012345678901234567890'
					},
					status: 'active',
					schedulerState: 'leader',
					error: 'This error should not appear'
				}
			]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				orderbookStatuses
			}
		});

		expect(screen.queryByText('This error should not appear')).not.toBeInTheDocument();
	});

	it('does not show phase message when schedulerState is notLeader even if status is syncing', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'syncing', schedulerState: 'notLeader' }]
		]);
		const orderbookStatuses = new Map<string, OrderbookSyncStatus>([
			[
				'137:0x1234567890123456789012345678901234567890',
				{
					obId: {
						chainId: 137,
						orderbookAddress: '0x1234567890123456789012345678901234567890'
					},
					status: 'syncing',
					schedulerState: 'notLeader',
					phaseMessage: 'Fetching latest block'
				}
			]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				orderbookStatuses
			}
		});

		expect(screen.getByText('Observing')).toBeInTheDocument();
		expect(screen.queryByText('Fetching latest block')).not.toBeInTheDocument();
	});

	it('shows network head block when latestBlock is present', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'active', schedulerState: 'leader', latestBlock: 50000000 }]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				orderbookStatuses: new Map()
			}
		});

		const headEl = screen.getByTestId('network-head-137');
		expect(headEl).toBeInTheDocument();
		expect(headEl.textContent).toContain('Head:');
		expect(headEl.textContent).toContain('50,000,000');
	});

	it('does not show network head block when latestBlock is undefined', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'active', schedulerState: 'leader' }]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				orderbookStatuses: new Map()
			}
		});

		expect(screen.queryByTestId('network-head-137')).not.toBeInTheDocument();
	});

	it('shows behind count when orderbook is syncing', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'syncing', schedulerState: 'leader' }]
		]);
		const orderbookStatuses = new Map<string, OrderbookSyncStatus>([
			[
				'137:0x1234567890123456789012345678901234567890',
				{
					obId: {
						chainId: 137,
						orderbookAddress: '0x1234567890123456789012345678901234567890'
					},
					status: 'syncing',
					schedulerState: 'leader',
					latestBlock: 1000,
					syncedBlock: 950
				}
			]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				orderbookStatuses
			}
		});

		const progressEl = screen.getByTestId('ob-block-progress');
		expect(progressEl).toBeInTheDocument();
		expect(progressEl.textContent).toContain('Synced to #950');
		expect(progressEl.textContent).toContain('50 behind');
	});

	it('does not show behind count when orderbook is active', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'active', schedulerState: 'leader' }]
		]);
		const orderbookStatuses = new Map<string, OrderbookSyncStatus>([
			[
				'137:0x1234567890123456789012345678901234567890',
				{
					obId: {
						chainId: 137,
						orderbookAddress: '0x1234567890123456789012345678901234567890'
					},
					status: 'active',
					schedulerState: 'leader',
					latestBlock: 1000,
					syncedBlock: 950
				}
			]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				orderbookStatuses
			}
		});

		const progressEl = screen.getByTestId('ob-block-progress');
		expect(progressEl).toBeInTheDocument();
		expect(progressEl.textContent).toContain('Synced to #950');
		expect(progressEl.textContent).not.toContain('behind');
	});

	it('does not show block progress when syncedBlock is undefined', () => {
		const networkStatuses = new Map<number, NetworkSyncStatus>([
			[137, { chainId: 137, status: 'active', schedulerState: 'leader' }]
		]);
		const orderbookStatuses = new Map<string, OrderbookSyncStatus>([
			[
				'137:0x1234567890123456789012345678901234567890',
				{
					obId: {
						chainId: 137,
						orderbookAddress: '0x1234567890123456789012345678901234567890'
					},
					status: 'active',
					schedulerState: 'leader'
				}
			]
		]);

		render(LocalDbStatusModal, {
			props: {
				open: true,
				networkStatuses,
				orderbookStatuses
			}
		});

		expect(screen.queryByTestId('ob-block-progress')).not.toBeInTheDocument();
	});

});
