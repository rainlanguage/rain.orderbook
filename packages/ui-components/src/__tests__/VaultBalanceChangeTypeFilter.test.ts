import { render, screen, fireEvent } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import VaultBalanceChangeTypeFilter from '../lib/components/VaultBalanceChangeTypeFilter.svelte';

describe('VaultBalanceChangeTypeFilter', () => {
	it('renders the filter dropdown', () => {
		render(VaultBalanceChangeTypeFilter);

		expect(screen.getByText('Trade Type')).toBeInTheDocument();
		expect(screen.getByTestId('dropdown-checkbox-button')).toBeInTheDocument();
	});

	it('shows "Select items" when nothing selected', () => {
		render(VaultBalanceChangeTypeFilter);

		expect(screen.getByTestId('dropdown-checkbox-button')).toHaveTextContent('Select items');
	});

	it('renders all filter options in dropdown', async () => {
		render(VaultBalanceChangeTypeFilter);

		const button = screen.getByTestId('dropdown-checkbox-button');
		await fireEvent.click(button);

		const options = screen.getAllByTestId('dropdown-checkbox-option');
		expect(options.length).toBeGreaterThanOrEqual(4);

		expect(screen.getByText('Deposit')).toBeInTheDocument();
		expect(screen.getByText('Withdrawal')).toBeInTheDocument();
		expect(screen.getByText('Trade')).toBeInTheDocument();
		expect(screen.getByText('Clear Bounty')).toBeInTheDocument();
	});

	it('shows "Select items" when no filters selected', () => {
		render(VaultBalanceChangeTypeFilter);

		expect(screen.getByTestId('dropdown-checkbox-button')).toHaveTextContent('Select items');
	});

	it('shows "All types" when all filters selected', async () => {
		render(VaultBalanceChangeTypeFilter);

		const button = screen.getByTestId('dropdown-checkbox-button');
		await fireEvent.click(button);

		const allTypesCheckbox = screen.getByText('All types').closest('label');
		if (allTypesCheckbox) {
			await fireEvent.click(allTypesCheckbox);
		}

		expect(screen.getByTestId('dropdown-checkbox-button')).toHaveTextContent('All types');
	});

	it('shows item count when some filters selected', async () => {
		render(VaultBalanceChangeTypeFilter);

		const button = screen.getByTestId('dropdown-checkbox-button');
		await fireEvent.click(button);

		const withdrawalLabel = screen.getByText('Withdrawal').closest('label');
		const depositLabel = screen.getByText('Deposit').closest('label');

		if (withdrawalLabel) await fireEvent.click(withdrawalLabel);
		if (depositLabel) await fireEvent.click(depositLabel);

		expect(screen.getByTestId('dropdown-checkbox-button')).toHaveTextContent('2 items');
	});
});
