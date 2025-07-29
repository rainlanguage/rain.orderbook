import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import InputTokenAmount from '$lib/components/input/InputTokenAmount.svelte';

describe('InputTokenAmount', () => {
	it('should handle numeric input', async () => {
		const { getByRole, component } = render(InputTokenAmount, {
			props: { value: '0' }
		});
		const input = getByRole('textbox');

		await fireEvent.input(input, { target: { value: '9999' } });
		expect(component.$$.ctx[component.$$.props.value]).toBe('9999');
	});

	it('should handle empty input', async () => {
		const { getByRole, component } = render(InputTokenAmount, {
			props: { value: '0' }
		});
		const input = getByRole('textbox');

		await fireEvent.input(input, { target: { value: '' } });
		expect(component.$$.ctx[component.$$.props.value]).toBe('0');
	});

	it('should handle invalid input', async () => {
		const { getByRole, component } = render(InputTokenAmount, {
			props: { value: '0' }
		});
		const input = getByRole('textbox');

		await fireEvent.input(input, { target: { value: 'abc' } });
		expect(component.$$.ctx[component.$$.props.value]).toBe('0');
	});

	it('should handle maxValue prop', async () => {
		const { getByText, component } = render(InputTokenAmount, {
			props: { maxValue: '1000', value: '0' }
		});
		const maxButton = getByText('MAX');

		await fireEvent.click(maxButton);
		expect(component.$$.ctx[component.$$.props.value]).toBe('1000');
	});
});
