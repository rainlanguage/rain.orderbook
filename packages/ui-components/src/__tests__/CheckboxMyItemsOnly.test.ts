import { render, fireEvent, screen } from '@testing-library/svelte';
import { get, writable, type Writable } from 'svelte/store';
import { beforeEach, expect, test, describe } from 'vitest';
import CheckboxMyItemsOnly from '../lib/components/CheckboxMyItemsOnly.svelte';

describe('CheckboxMyItemsOnly', () => {
	let showMyItemsOnly: Writable<boolean>;
	let context: 'orders' | 'vaults';
	beforeEach(() => {
		showMyItemsOnly = writable(true);
		context = 'orders';
	});

	test('renders correctly on orders page', () => {
		render(CheckboxMyItemsOnly, {
			props: {
				showMyItemsOnly,
				context
			}
		});
		expect(screen.getByText('Only show my orders')).toBeInTheDocument();
	});
	test('renders correctly on vaults page', () => {
		render(CheckboxMyItemsOnly, {
			props: {
				showMyItemsOnly,
				context: 'vaults'
			}
		});
		expect(screen.getByText('Only show my vaults')).toBeInTheDocument();
	});

	test('toggles store value when clicked', async () => {
		render(CheckboxMyItemsOnly, {
			props: {
				showMyItemsOnly,
				context
			}
		});

		const checkbox = screen.getByRole('checkbox');
		expect(get(showMyItemsOnly)).toBe(true);

		await fireEvent.click(checkbox);
		expect(get(showMyItemsOnly)).toBe(false);
		//
		await fireEvent.click(checkbox);
		expect(get(showMyItemsOnly)).toBe(true);
	});
});
