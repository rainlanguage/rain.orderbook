import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { get, writable, type Writable } from 'svelte/store';
import { beforeEach, expect, test, describe } from 'vitest';
import DropdownOrderStatus from '../lib/components/dropdown/DropdownOrderStatus.svelte';

describe('DropdownOrderStatus', () => {
	let activeOrderStatus: Writable<boolean | undefined>;

	beforeEach(() => {
		activeOrderStatus = writable(undefined);
	});

	test('renders correctly', () => {
		render(DropdownOrderStatus, {
			props: {
				activeOrderStatus
			}
		});
		expect(screen.getByText('Status')).toBeInTheDocument();
	});

	test('displays the correct number of options', async () => {
		render(DropdownOrderStatus, {
			props: {
				activeOrderStatus
			}
		});

		await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));

		await waitFor(() => {
			const options = screen.getAllByTestId('dropdown-checkbox-option');
			expect(options).toHaveLength(2); // Active and Inactive options
		});
	});

	test('updates status when Active is selected', async () => {
		render(DropdownOrderStatus, {
			props: {
				activeOrderStatus
			}
		});

		await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));
		await fireEvent.click(screen.getByText('Active'));

		await waitFor(() => {
			expect(get(activeOrderStatus)).toBe(true);
		});
	});

	test('updates status when Inactive is selected', async () => {
		render(DropdownOrderStatus, {
			props: {
				activeOrderStatus
			}
		});

		await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));
		await fireEvent.click(screen.getByText('Inactive'));

		await waitFor(() => {
			expect(get(activeOrderStatus)).toBe(false);
		});
	});

	test('resets to undefined when both options are selected', async () => {
		render(DropdownOrderStatus, {
			props: {
				activeOrderStatus
			}
		});

		await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));
		await fireEvent.click(screen.getByText('Active'));
		await fireEvent.click(screen.getByText('Inactive'));

		await waitFor(() => {
			expect(get(activeOrderStatus)).toBe(undefined);
		});
	});

	test('resets to undefined when no options are selected', async () => {
		activeOrderStatus.set(true);

		render(DropdownOrderStatus, {
			props: {
				activeOrderStatus
			}
		});

		await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));
		await fireEvent.click(screen.getByText('Active'));

		await waitFor(() => {
			expect(get(activeOrderStatus)).toBe(undefined);
		});
	});
});
