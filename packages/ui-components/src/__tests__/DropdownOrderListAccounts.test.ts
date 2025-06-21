import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { get, writable, type Writable } from 'svelte/store';
import DropdownOrderListAccounts from '../lib/components/dropdown/DropdownOrderListAccounts.svelte';
import { expect, test, describe, beforeEach } from 'vitest';
import type { AccountCfg } from '@rainlanguage/orderbook';

describe('DropdownOrderListAccounts', () => {
	let accounts: Writable<Record<string, AccountCfg>>;
	let activeAccountsItems: Writable<Record<string, string>>;

	beforeEach(() => {
		accounts = writable({
			address1: {
				key: 'address1',
				address: 'Label 1'
			},
			address2: {
				key: 'address2',
				address: 'Label 2'
			},
			address3: {
				key: 'address3',
				address: 'Label 3'
			}
		});
		activeAccountsItems = writable({});
	});

	test('renders correctly', () => {
		render(DropdownOrderListAccounts, {
			props: {
				accounts,
				activeAccountsItems
			}
		});
		expect(screen.getByText('Accounts')).toBeInTheDocument();
	});

	test('displays the correct number of options', async () => {
		render(DropdownOrderListAccounts, {
			props: {
				accounts,
				activeAccountsItems
			}
		});

		await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));

		await waitFor(() => {
			const options = screen.getAllByTestId('dropdown-checkbox-option');
			expect(options).toHaveLength(4); // Including "All accounts"
		});
	});

	test('updates active accounts when an option is selected', async () => {
		render(DropdownOrderListAccounts, {
			props: {
				accounts,
				activeAccountsItems
			}
		});

		await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));
		await fireEvent.click(screen.getByText('Label 1'));

		await waitFor(() => {
			expect(get(activeAccountsItems)).toEqual({ address1: 'Label 1' });
		});
	});

	test('selects all items when "All accounts" is clicked', async () => {
		render(DropdownOrderListAccounts, {
			props: {
				accounts,
				activeAccountsItems
			}
		});

		await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));
		await fireEvent.click(screen.getByText('All accounts'));

		await waitFor(() => {
			expect(get(activeAccountsItems)).toEqual({
				address1: 'Label 1',
				address2: 'Label 2',
				address3: 'Label 3'
			});
		});
	});

	test('displays "No accounts added" when accounts list is empty', async () => {
		accounts.set({});

		render(DropdownOrderListAccounts, {
			props: {
				accounts,
				activeAccountsItems
			}
		});

		await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));

		await waitFor(() => {
			expect(screen.getByText('No accounts added')).toBeInTheDocument();
		});
	});
});
