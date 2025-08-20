import { render, fireEvent, screen } from '@testing-library/svelte';
import { expect, test, describe, vi } from 'vitest';
import CheckboxZeroBalanceVault from '../lib/components/CheckboxZeroBalanceVault.svelte';

describe('CheckboxZeroBalanceVault', () => {
	test('renders correctly', () => {
		const onChange = vi.fn();
		render(CheckboxZeroBalanceVault, {
			props: {
				checked: false,
				onChange
			}
		});
		expect(screen.getByText('Hide empty vaults')).toBeInTheDocument();
	});

	test('calls onChange when clicked', async () => {
		const onChange = vi.fn();
		render(CheckboxZeroBalanceVault, {
			props: {
				checked: false,
				onChange
			}
		});

		const checkbox = screen.getByRole('checkbox');
		expect(checkbox).not.toBeChecked();

		await fireEvent.click(checkbox);
		expect(onChange).toHaveBeenCalledWith(true);

		// Test checked state
		render(CheckboxZeroBalanceVault, {
			props: {
				checked: true,
				onChange
			}
		});

		const checkedBox = screen.getByRole('checkbox');
		expect(checkedBox).toBeChecked();

		await fireEvent.click(checkedBox);
		expect(onChange).toHaveBeenCalledWith(false);
	});
});
