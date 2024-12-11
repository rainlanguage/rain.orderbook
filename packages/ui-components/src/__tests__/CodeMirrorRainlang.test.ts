import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '@testing-library/svelte';
import CodeMirrorRainlang from './CodeMirrorRainlang.test.svelte';
import type { Order } from '@rainlanguage/orderbook/js_api';
import * as orderBookApi from '@rainlanguage/orderbook/js_api';
import { writable } from 'svelte/store';

// Mock the extendOrder function
vi.mock('@rainlanguage/orderbook/js_api', () => ({
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	extendOrder: vi.fn((order: Order) => ({
		rainlang: 'mocked rainlang text'
	}))
}));

describe('CodeMirrorRainlang', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should use extendOrder when order prop is provided', () => {
		const mockOrder: Order = {} as Order;

		const { getByTestId } = render(CodeMirrorRainlang, {
			props: {
				props: {
					order: mockOrder,
					rainlangText: 'original text',
					codeMirrorTheme: writable({})
				}
			}
		});

		expect(orderBookApi.extendOrder).toHaveBeenCalledWith(mockOrder);
		expect(getByTestId('test-value').textContent).toBe('mocked rainlang text');
	});

	it('should use rainlangText when no order is provided', () => {
		const testText = 'test rainlang text';

		const { getByTestId } = render(CodeMirrorRainlang, {
			props: {
				props: {
					rainlangText: testText,
					codeMirrorTheme: writable({})
				}
			}
		});

		expect(orderBookApi.extendOrder).not.toHaveBeenCalled();
		expect(getByTestId('test-value').textContent).toBe(testText);
	});

	it('should pass through disabled prop', () => {
		const { getByTestId } = render(CodeMirrorRainlang, {
			props: {
				props: {
					disabled: true,
					rainlangText: 'test',
					codeMirrorTheme: writable({})
				}
			}
		});

		expect(getByTestId('test-value')).toBeTruthy();
	});
});
