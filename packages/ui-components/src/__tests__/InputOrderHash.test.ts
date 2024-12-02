import { render, fireEvent } from '@testing-library/svelte';
import { writable, get } from 'svelte/store';
import InputOrderHash from '../lib/components/input/InputOrderHash.svelte';

describe('InputOrderHash', () => {
	it('renders with initial value', () => {
		const orderHash = writable('0x123');
		const { getByTestId } = render(InputOrderHash, { props: { orderHash } });

		const input = getByTestId('order-hash-input').querySelector('input');
		expect(input?.value).toBe('0x123');
	});

	it('updates store value when input changes', async () => {
		const orderHash = writable('');
		const { getByTestId } = render(InputOrderHash, { props: { orderHash } });

		const input = getByTestId('order-hash-input').querySelector('input') as HTMLInputElement;
		await fireEvent.input(input, { target: { value: '0xabc' } });

		expect(input.value).toBe('0xabc');
		expect(get(orderHash)).toBe('0xabc');
	});

	it('renders with placeholder', () => {
		const orderHash = writable('');
		const { getByTestId } = render(InputOrderHash, { props: { orderHash } });

		const input = getByTestId('order-hash-input').querySelector('input');
		expect(input?.placeholder).toBe('0x...');
	});
});
