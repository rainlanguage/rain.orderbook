import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, waitFor } from '@testing-library/svelte';
import CodeMirrorRainlang from '../lib/components/CodeMirrorRainlang.svelte';
import type { SgOrder } from '@rainlanguage/orderbook';
import { extendOrder } from '@rainlanguage/orderbook';
import { writable } from 'svelte/store';

// Mock the extendOrder function
vi.mock('@rainlanguage/orderbook', () => ({
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	extendOrder: vi.fn((order: SgOrder) => ({
		rainlang: 'mocked rainlang text'
	}))
}));

vi.mock('codemirror-rainlang', () => ({
	RainlangLR: vi.fn()
}));

vi.mock('svelte-codemirror-editor', async () => {
	const mockCodeMirror = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return { default: mockCodeMirror };
});

describe('CodeMirrorRainlang', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should use extendOrder when order prop is provided', () => {
		const mockOrder: SgOrder = {} as SgOrder;

		render(CodeMirrorRainlang, {
			props: {
				order: mockOrder,
				codeMirrorTheme: writable({}),
				codeMirrorDisabled: false,
				codeMirrorStyles: {}
			}
		});

		expect(extendOrder).toHaveBeenCalledWith(mockOrder);
	});

	it('should use rainlangText when no order is provided', () => {
		const testText = 'test rainlang text';

		render(CodeMirrorRainlang, {
			props: {
				order: undefined,
				rainlangText: testText,
				codeMirrorTheme: writable({}),
				codeMirrorDisabled: false,
				codeMirrorStyles: {}
			}
		});

		expect(extendOrder).not.toHaveBeenCalled();
	});

	it('should pass through disabled prop', async () => {
		const mockOrder: SgOrder = {} as SgOrder;
		const mockExtendedOrder = { order: {} };

		(extendOrder as Mock).mockReturnValue(mockExtendedOrder);

		const screen = render(CodeMirrorRainlang, {
			props: {
				order: mockOrder,
				codeMirrorTheme: writable({}),
				codeMirrorDisabled: false,
				codeMirrorStyles: {}
			}
		});

		await waitFor(() => {
			expect(screen.getByTestId('rainlang-not-included')).toBeInTheDocument();
		});
	});
});
