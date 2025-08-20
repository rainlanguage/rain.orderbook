import { render, fireEvent } from '@testing-library/svelte';
import { vi } from 'vitest';
import InputOrderHash from '../lib/components/input/InputOrderHash.svelte';

describe('InputOrderHash', () => {
	it('renders with initial value', () => {
		const onChange = vi.fn();
		const { getByTestId } = render(InputOrderHash, {
			props: {
				value: '0x0123',
				onChange
			}
		});

		const input = getByTestId('order-hash-input').querySelector('input');
		expect(input?.value).toBe('0x0123');
	});

	it('calls onChange when input changes', async () => {
		const onChange = vi.fn();
		const { getByTestId } = render(InputOrderHash, {
			props: {
				value: '0x0123',
				onChange
			}
		});

		const input = getByTestId('order-hash-input').querySelector('input') as HTMLInputElement;
		await fireEvent.input(input, { target: { value: '0xabc' } });

		expect(input.value).toBe('0xabc');
		expect(onChange).toHaveBeenCalledWith('0xabc');
	});

	it('renders with placeholder', () => {
		const onChange = vi.fn();
		const { getByTestId } = render(InputOrderHash, {
			props: {
				value: '0x0345',
				onChange
			}
		});

		const input = getByTestId('order-hash-input').querySelector('input');
		expect(input?.placeholder).toBe('0x...');
	});
});
