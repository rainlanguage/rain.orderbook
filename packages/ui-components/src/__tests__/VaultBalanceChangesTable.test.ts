import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '../lib/test/matchers';
import { QueryClient } from '@tanstack/svelte-query';
import VaultBalanceChangesTable from '../lib/components/tables/VaultBalanceChangesTable.svelte';
import type { RaindexVault, RaindexVaultBalanceChange } from '@rainlanguage/orderbook';
import { formatTimestampSecondsAsLocal } from '../lib/services/time';

test('renders the vault list table with correct data', async () => {
	const queryClient = new QueryClient();

	const mockVaultBalanceChanges: RaindexVaultBalanceChange[] = [
		{
			__typename: 'Withdrawal',
			amount: BigInt(1000),
			oldBalance: BigInt(5000),
			newBalance: BigInt(4000),
			timestamp: BigInt(1625247600),
			vaultId: BigInt(100),
			token: {
				id: 'token1',
				address: '0xTokenAddress1',
				name: 'Token1',
				symbol: 'TKN1',
				decimals: '18'
			},
			transaction: {
				id: 'tx1',
				from: '0xUser1',
				timestamp: BigInt(1625247600),
				blockNumber: BigInt(1234567890)
			},
			orderbook: '0x00'
		}
		// ... other mock data
	] as unknown as RaindexVaultBalanceChange[];
	const mockVault: RaindexVault = {
		getBalanceChanges: vi.fn().mockResolvedValue({ value: mockVaultBalanceChanges })
	} as unknown as RaindexVault;

	render(VaultBalanceChangesTable, {
		props: {
			id: '100',
			vault: mockVault
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

	const mockVaultBalanceChanges: RaindexVaultBalanceChange[] = [
		{
			__typename: 'Withdrawal',
			amount: BigInt(1000),
			oldBalance: BigInt(5000),
			newBalance: BigInt(4000),
			timestamp: BigInt(1625247600),
			vaultId: BigInt(100),
			token: {
				id: 'token1',
				address: '0xTokenAddress1',
				name: 'Token1',
				symbol: 'TKN1',
				decimals: '4'
			},
			transaction: {
				id: 'tx1',
				from: '0xUser1',
				timestamp: BigInt(1625247600),
				blockNumber: BigInt(1234567890)
			},
			orderbook: '0x00'
		}
	] as unknown as RaindexVaultBalanceChange[];
	const mockVault: RaindexVault = {
		getBalanceChanges: vi.fn().mockResolvedValue({ value: mockVaultBalanceChanges })
	} as unknown as RaindexVault;

	render(VaultBalanceChangesTable, {
		props: {
			id: '100',
			vault: mockVault
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
