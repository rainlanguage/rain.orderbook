import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import DropdownOrderListAccounts from '../lib/components/dropdown/DropdownOrderListAccounts.svelte';
import { expect, test, describe, type Mock, vi } from 'vitest';
import { useRaindexClient } from '$lib/hooks/useRaindexClient';

vi.mock('$lib/hooks/useRaindexClient', () => ({
	useRaindexClient: vi.fn()
}));

describe('DropdownOrderListAccounts', () => {
	const mockUseRaindexClient = () => {
		(useRaindexClient as Mock).mockReturnValue({
			getAllAccounts: vi.fn().mockReturnValue({
				value: new Map([
					['address1', '0x1234567890123456789012345678901234567890'],
					['address2', '0x1234567890123456789012345678901234567891'],
					['address3', '0x1234567890123456789012345678901234567892']
				])
			})
		});
	};

	test('renders correctly', () => {
		mockUseRaindexClient();
		const onChange = vi.fn();

		render(DropdownOrderListAccounts, {
			props: {
				activeAccountsItems: {},
				onChange
			}
		});
		expect(screen.getByText('Accounts')).toBeInTheDocument();
	});

	test('displays the correct number of options', async () => {
		mockUseRaindexClient();
		const onChange = vi.fn();

		render(DropdownOrderListAccounts, {
			props: {
				activeAccountsItems: {},
				onChange
			}
		});

		await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));

		await waitFor(() => {
			const options = screen.getAllByTestId('dropdown-checkbox-option');
			expect(options).toHaveLength(4); // Including "All accounts"
		});
	});

	test('calls onChange when an option is selected', async () => {
		mockUseRaindexClient();
		const onChange = vi.fn();

		render(DropdownOrderListAccounts, {
			props: {
				activeAccountsItems: {},
				onChange
			}
		});

		await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));
		await fireEvent.click(screen.getByText('0x1234567890123456789012345678901234567890'));

		await waitFor(() => {
			expect(onChange).toHaveBeenCalledWith({
				address1: '0x1234567890123456789012345678901234567890'
			});
		});
	});

	test('displays "No accounts added" when accounts list is empty', async () => {
		(useRaindexClient as Mock).mockReturnValue({
			getAllAccounts: vi.fn().mockReturnValue({
				value: new Map()
			})
		});

		const onChange = vi.fn();

		render(DropdownOrderListAccounts, {
			props: {
				activeAccountsItems: {},
				onChange
			}
		});

		await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));

		await waitFor(() => {
			expect(screen.getByText('No accounts added')).toBeInTheDocument();
		});
	});
});
