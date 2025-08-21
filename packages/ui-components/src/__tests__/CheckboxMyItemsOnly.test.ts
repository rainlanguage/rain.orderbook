import { render, fireEvent, screen } from '@testing-library/svelte';
import { expect, test, describe, vi } from 'vitest';
import CheckboxMyItemsOnly from '../lib/components/CheckboxMyItemsOnly.svelte';

// Mock useAccount
vi.mock('$lib/providers/wallet/useAccount', () => ({
	useAccount: () => ({
		account: {
			subscribe: (fn: (value: string) => void) => {
				fn('0x1234567890123456789012345678901234567890');
				return { unsubscribe: () => {} };
			}
		}
	})
}));

describe('CheckboxMyItemsOnly', () => {
	test('renders correctly on orders page', () => {
		const onChange = vi.fn();
		render(CheckboxMyItemsOnly, {
			props: {
				currentOwners: ['0x1234567890123456789012345678901234567890'],
				onChange,
				context: 'orders'
			}
		});
		expect(screen.getByText('Only show my orders')).toBeInTheDocument();
	});

	test('renders correctly on vaults page', () => {
		const onChange = vi.fn();
		render(CheckboxMyItemsOnly, {
			props: {
				currentOwners: ['0x1234567890123456789012345678901234567890'],
				onChange,
				context: 'vaults'
			}
		});
		expect(screen.getByText('Only show my vaults')).toBeInTheDocument();
	});

	test('calls onChange when clicked', async () => {
		const onChange = vi.fn();
		render(CheckboxMyItemsOnly, {
			props: {
				currentOwners: ['0x1234567890123456789012345678901234567890'],
				onChange,
				context: 'orders'
			}
		});

		const checkbox = screen.getByRole('checkbox');
		expect(checkbox).toBeChecked(); // Should be checked because current account is in owners

		await fireEvent.click(checkbox);
		expect(onChange).toHaveBeenCalledWith(false);
	});

	test('calls onChange when clicked from unchecked state', async () => {
		const onChange = vi.fn();
		render(CheckboxMyItemsOnly, {
			props: {
				currentOwners: [], // Empty owners
				onChange,
				context: 'orders'
			}
		});

		const uncheckedBox = screen.getByRole('checkbox');
		expect(uncheckedBox).not.toBeChecked();

		await fireEvent.click(uncheckedBox);
		expect(onChange).toHaveBeenCalledWith(true);
	});
});
