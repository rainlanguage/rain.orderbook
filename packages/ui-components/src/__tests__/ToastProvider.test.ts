import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import ToastProvider from '../lib/providers/toasts/ToastProvider.svelte';
import { writable } from 'svelte/store';
import type { ToastProps } from '../lib/types/toast';

vi.mock('../lib/providers/toasts/context', () => ({
	setToastsContext: vi.fn()
}));

import { setToastsContext } from '../lib/providers/toasts/context';

describe('ToastProvider', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should call setToastsContext with the toasts prop', () => {
		const mockToasts = writable<ToastProps[]>([]);

		render(ToastProvider, {
			props: {
				toasts: mockToasts
			}
		});

		expect(setToastsContext).toHaveBeenCalledWith(mockToasts);
	});

	it('should use default empty toasts array when no toasts are provided', () => {
		render(ToastProvider);

		expect(setToastsContext).toHaveBeenCalled();
		const toastsArg = vi.mocked(setToastsContext).mock.calls[0][0];
		expect(toastsArg).toBeDefined();

		let value;
		toastsArg.subscribe((v) => {
			value = v;
		})();
		expect(value).toEqual([]);
	});

	it('should properly render toast notifications', () => {
		const mockToasts = writable<ToastProps[]>([
			{ color: 'green', message: 'Test toast', type: 'success' }
		]);

		render(ToastProvider, {
			props: {
				toasts: mockToasts
			}
		});

		expect(screen.getByRole('alert')).toBeInTheDocument();
	});
});
