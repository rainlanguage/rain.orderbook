import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi, describe } from 'vitest';
import { expect } from '../lib/test/matchers';
import { QueryClient } from '@tanstack/svelte-query';
import VaultBalanceChangesTable from '../lib/components/tables/VaultBalanceChangesTable.svelte';
import type { RaindexVault, RaindexVaultBalanceChange } from '@rainlanguage/orderbook';
import { formatTimestampSecondsAsLocal } from '../lib/services/time';

const createMockVaultBalanceChange = (
	overrides: Partial<RaindexVaultBalanceChange> = {}
): RaindexVaultBalanceChange =>
	({
		type: 'withdrawal',
		amount: BigInt(1000),
		formattedAmount: '0.1',
		oldBalance: BigInt(5000),
		formattedOldBalance: '0.5',
		newBalance: BigInt(4000),
		formattedNewBalance: '0.4',
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
		orderbook: '0x00',
		...overrides
	}) as unknown as RaindexVaultBalanceChange;

test('renders the vault list table with correct data', async () => {
	const queryClient = new QueryClient();

	const mockVaultBalanceChanges: RaindexVaultBalanceChange[] = [createMockVaultBalanceChange()];
	const mockVault: RaindexVault = {
		id: 'vault1',
		getBalanceChanges: vi.fn().mockResolvedValue({ value: mockVaultBalanceChanges })
	} as unknown as RaindexVault;

	render(VaultBalanceChangesTable, {
		props: {
			vault: mockVault
		},
		context: new Map([['$$_queryClient', queryClient]])
	});

	await waitFor(() => {
		const rows = screen.getAllByTestId('bodyRow');
		expect(rows).toHaveLength(1);
	});
});

test('it shows the correct data in the table with combined Info column', async () => {
	const queryClient = new QueryClient();

	const mockVaultBalanceChanges: RaindexVaultBalanceChange[] = [createMockVaultBalanceChange()];
	const mockVault: RaindexVault = {
		id: 'vault1',
		getBalanceChanges: vi.fn().mockResolvedValue({ value: mockVaultBalanceChanges })
	} as unknown as RaindexVault;

	render(VaultBalanceChangesTable, {
		props: {
			vault: mockVault
		},
		context: new Map([['$$_queryClient', queryClient]])
	});

	await waitFor(() => {
		// Info column now contains both type badge and date
		const infoCell = screen.getByTestId('vaultBalanceChangesTableInfo');
		expect(infoCell).toHaveTextContent('Withdrawal');
		expect(infoCell).toHaveTextContent(formatTimestampSecondsAsLocal(BigInt('1625247600')));

		// Transaction column now contains both Sender and Tx
		const txCell = screen.getByTestId('vaultBalanceChangesTableTx');
		expect(txCell).toHaveTextContent('Sender:');
		expect(txCell).toHaveTextContent('Tx:');

		// Balance Change column shows token symbol and amount
		const balanceChangeCell = screen.getByTestId('vaultBalanceChangesTableBalanceChange');
		expect(balanceChangeCell).toHaveTextContent('TKN1');
		expect(balanceChangeCell).toHaveTextContent('0.1');

		// New Balance column shows token symbol and amount
		const balanceCell = screen.getByTestId('vaultBalanceChangesTableBalance');
		expect(balanceCell).toHaveTextContent('TKN1');
		expect(balanceCell).toHaveTextContent('0.4');
	});
});

describe('type badge colors', () => {
	const testCases = [
		{ type: 'deposit', expectedColor: 'green', expectedLabel: 'Deposit' },
		{ type: 'withdrawal', expectedColor: 'yellow', expectedLabel: 'Withdrawal' },
		{
			type: 'tradeVaultBalanceChange',
			expectedColor: 'blue',
			expectedLabel: 'Trade Vault Balance Change'
		},
		{ type: 'clearBounty', expectedColor: 'purple', expectedLabel: 'Clear Bounty' }
	];

	testCases.forEach(({ type, expectedLabel }) => {
		test(`displays ${type} with correct label`, async () => {
			const queryClient = new QueryClient();

			const mockVaultBalanceChanges: RaindexVaultBalanceChange[] = [
				createMockVaultBalanceChange({ type })
			];
			const mockVault: RaindexVault = {
				id: 'vault1',
				getBalanceChanges: vi.fn().mockResolvedValue({ value: mockVaultBalanceChanges })
			} as unknown as RaindexVault;

			render(VaultBalanceChangesTable, {
				props: {
					vault: mockVault
				},
				context: new Map([['$$_queryClient', queryClient]])
			});

			await waitFor(() => {
				const infoCell = screen.getByTestId('vaultBalanceChangesTableInfo');
				expect(infoCell).toHaveTextContent(expectedLabel);
			});
		});
	});
});

test('formats camelCase types correctly', async () => {
	const queryClient = new QueryClient();

	const mockVaultBalanceChanges: RaindexVaultBalanceChange[] = [
		createMockVaultBalanceChange({ type: 'someComplexTypeName' })
	];
	const mockVault: RaindexVault = {
		id: 'vault1',
		getBalanceChanges: vi.fn().mockResolvedValue({ value: mockVaultBalanceChanges })
	} as unknown as RaindexVault;

	render(VaultBalanceChangesTable, {
		props: {
			vault: mockVault
		},
		context: new Map([['$$_queryClient', queryClient]])
	});

	await waitFor(() => {
		const infoCell = screen.getByTestId('vaultBalanceChangesTableInfo');
		expect(infoCell).toHaveTextContent('Some Complex Type Name');
	});
});
