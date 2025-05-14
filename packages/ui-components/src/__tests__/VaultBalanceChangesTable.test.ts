import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi, type Mock } from 'vitest';
import { expect } from '../lib/test/matchers';
import { QueryClient } from '@tanstack/svelte-query';
import VaultBalanceChangesTable from '../lib/components/tables/VaultBalanceChangesTable.svelte';
import type { SgVaultBalanceChangeType } from '@rainlanguage/orderbook';
import { formatTimestampSecondsAsLocal } from '../lib/services/time';

vi.mock('@rainlanguage/orderbook', () => ({
	getVaultBalanceChanges: vi.fn()
}));

test('renders the vault list table with correct data', async () => {
	const queryClient = new QueryClient();

	const mockVaultBalanceChanges: SgVaultBalanceChangeType[] = [
		{
			__typename: 'Withdrawal',
			amount: '1000',
			oldVaultBalance: '5000',
			newVaultBalance: '4000',
			vault: {
				id: 'vault1',
				vault_id: 'vault-id1',
				token: {
					id: 'token1',
					address: '0xTokenAddress1',
					name: 'Token1',
					symbol: 'TKN1',
					decimals: '18'
				}
			},
			timestamp: '1625247600',
			transaction: {
				id: 'tx1',
				from: '0xUser1',
				timestamp: '0',
				blockNumber: '0'
			},
			orderbook: {
				id: '0x00'
			}
		}
		// ... other mock data
	] as unknown as SgVaultBalanceChangeType[];

	// Mock the getVaultBalanceChanges function
	const { getVaultBalanceChanges } = await import('@rainlanguage/orderbook');
	(getVaultBalanceChanges as Mock).mockResolvedValue({ value: mockVaultBalanceChanges });

	render(VaultBalanceChangesTable, {
		props: {
			id: '100',
			subgraphUrl: 'https://example.com'
		},
		context: new Map([['$$_queryClient', queryClient]])
	});

	await waitFor(() => {
		const rows = screen.getAllByTestId('bodyRow');
		expect(rows).toHaveLength(1);
	});
});

test('it shows the correct data in the table', async () => {
	const queryClient = new QueryClient();

	const mockVaultBalanceChanges: SgVaultBalanceChangeType[] = [
		{
			__typename: 'Withdrawal',
			amount: '1000',
			oldVaultBalance: '5000',
			newVaultBalance: '4000',
			vault: {
				id: 'vault1',
				vault_id: 'vault-id1',
				token: {
					id: 'token1',
					address: '0xTokenAddress1',
					name: 'Token1',
					symbol: 'TKN1',
					decimals: '4'
				}
			},
			timestamp: '1625247600',
			transaction: {
				id: 'tx1',
				from: '0xUser1',
				timestamp: '0',
				blockNumber: '0'
			},
			orderbook: {
				id: '0x00'
			}
		}
	] as unknown as SgVaultBalanceChangeType[];

	// Mock the getVaultBalanceChanges function
	const { getVaultBalanceChanges } = await import('@rainlanguage/orderbook');
	(getVaultBalanceChanges as Mock).mockResolvedValue({ value: mockVaultBalanceChanges });

	render(VaultBalanceChangesTable, {
		props: {
			id: '100',
			subgraphUrl: 'https://example.com'
		},
		context: new Map([['$$_queryClient', queryClient]])
	});

	await waitFor(() => {
		expect(screen.getByTestId('vaultBalanceChangesTableDate')).toHaveTextContent(
			formatTimestampSecondsAsLocal(BigInt('1625247600'))
		);
		expect(screen.getByTestId('vaultBalanceChangesTableFrom')).toHaveTextContent('0xUse...User1');
		expect(screen.getByTestId('vaultBalanceChangesTableTx')).toHaveTextContent('tx1');
		expect(screen.getByTestId('vaultBalanceChangesTableBalanceChange')).toHaveTextContent(
			'0.1 TKN1'
		);
		expect(screen.getByTestId('vaultBalanceChangesTableBalance')).toHaveTextContent('0.4 TKN1');
		expect(screen.getByTestId('vaultBalanceChangesTableType')).toHaveTextContent('Withdrawal');
	});
});
