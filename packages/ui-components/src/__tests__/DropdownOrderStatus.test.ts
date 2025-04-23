import { render, waitFor, screen } from '@testing-library/svelte';
import { describe, it, expect, beforeEach } from 'vitest';
import { get, writable } from 'svelte/store';
import type { Writable } from 'svelte/store';
import CheckboxOrderStatus from '../lib/components/checkbox/CheckboxActiveOrders.svelte'; // Adjust path as needed
import { userEvent } from '@testing-library/user-event';

describe('OrderStatusCheckbox Component', () => {
	let activeOrderStatus: Writable<boolean | undefined>;

	beforeEach(() => {
		// Create a fresh writable store for each test
		activeOrderStatus = writable(true);
	});

	it('renders the checkbox with correct label', () => {
		const { getByText, getByTestId } = render(CheckboxOrderStatus, {
			props: { activeOrderStatus }
		});

		expect(getByTestId('order-status-checkbox')).toBeTruthy();
		expect(getByText('Include inactive orders')).toBeTruthy();
	});

	it('initializes includeInactive based on activeOrderStatus store value', async () => {
		render(CheckboxOrderStatus, {
			props: { activeOrderStatus }
		});
		const checkbox = screen.getByTestId('order-status-checkbox');
		await userEvent.click(checkbox);

		await waitFor(() => {
			expect(get(activeOrderStatus)).toBe(undefined);
		});
	});

	it('updates activeOrderStatus store when checkbox is clicked', async () => {
		activeOrderStatus.set(undefined);

		render(CheckboxOrderStatus, {
			props: { activeOrderStatus }
		});

		const checkboxInput = screen.getByTestId('order-status-checkbox');
		await userEvent.click(checkboxInput);
		await waitFor(() => {
			expect(get(activeOrderStatus)).toBe(true);
		});
	});
});
