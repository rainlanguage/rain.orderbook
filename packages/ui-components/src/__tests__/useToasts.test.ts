import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { writable } from 'svelte/store';
import { useToasts } from '$lib/providers/toasts/useToasts';
import type { ToastProps } from '$lib/types/toast';
import { getToastsContext } from '$lib/providers/toasts/context';

vi.mock('$lib/providers/toasts/context', () => ({
	getToastsContext: vi.fn()
}));

describe('useToasts', () => {
	let toastsStore: ReturnType<typeof writable<ToastProps[]>>;

	const getStoreValue = () => {
		let value: ToastProps[] = [];
		toastsStore.subscribe((val) => {
			value = val;
		});
		return value;
	};

	beforeEach(() => {
		toastsStore = writable<ToastProps[]>([]);
		vi.mocked(getToastsContext).mockReturnValue(toastsStore);
	});

	afterEach(() => {
		vi.clearAllMocks();
	});

	it('should return the toasts store and functions', () => {
		const result = useToasts();

		expect(result.toasts).toBe(toastsStore);
		expect(typeof result.addToast).toBe('function');
		expect(typeof result.removeToast).toBe('function');
		expect(typeof result.errToast).toBe('function');
	});

	describe('errToast', () => {
		it('should add an error toast with the correct properties', () => {
			const { errToast } = useToasts();
			const errorMessage = 'Test error message';

			errToast(errorMessage);

			expect(getStoreValue()).toEqual([
				{
					message: errorMessage,
					type: 'error',
					color: 'red',
					id: 'mocked-uuid'
				}
			]);

			vi.advanceTimersByTime(3000);
			expect(getStoreValue()).toEqual([]);
		});
	});

	describe('addToast', () => {
		it('should add a toast to the store', () => {
			const { addToast } = useToasts();
			const testToast: ToastProps = {
				message: 'Test Toast',
				type: 'info',
				color: 'green',
				links: []
			};

			addToast(testToast);
			expect(getStoreValue()).toEqual([testToast]);
		});

		it('should add multiple toasts in sequence', () => {
			const { addToast } = useToasts();
			const toast1: ToastProps = {
				message: 'Toast 1',
				type: 'info',
				color: 'green',
				links: []
			};
			const toast2: ToastProps = {
				message: 'Toast 2',
				type: 'success',
				color: 'blue',
				links: []
			};

			addToast(toast1);
			addToast(toast2);

			expect(getStoreValue()).toEqual([toast1, toast2]);
		});

		it('should add a toast with minimal properties', () => {
			const { addToast } = useToasts();
			const testToast: ToastProps = {
				message: 'Test Toast Only Message',
				type: 'info',
				color: 'green',
				links: []
			};

			addToast(testToast);
			expect(getStoreValue()).toEqual([testToast]);
		});
	});

	describe('removeToast', () => {
		it('should remove a toast at the specified index', () => {
			const { addToast, removeToast } = useToasts();
			const toast1: ToastProps = {
				message: 'Toast 1',
				type: 'info',
				color: 'green',
				links: []
			};
			const toast2: ToastProps = {
				message: 'Toast 2',
				type: 'info',
				color: 'green',
				links: []
			};
			const toast3: ToastProps = {
				message: 'Toast 3',
				type: 'info',
				color: 'green',
				links: []
			};

			addToast(toast1);
			addToast(toast2);
			addToast(toast3);
			removeToast(1);

			expect(getStoreValue()).toEqual([toast1, toast3]);
		});

		it('should not modify the store when removing with an invalid index', () => {
			const { addToast, removeToast } = useToasts();
			const toast1: ToastProps = {
				message: 'Toast 1',
				type: 'info',
				color: 'green',
				links: []
			};
			const toast2: ToastProps = {
				message: 'Toast 2',
				type: 'info',
				color: 'green',
				links: []
			};

			addToast(toast1);
			addToast(toast2);

			// Test negative index
			removeToast(-1);
			expect(getStoreValue()).toEqual([toast1, toast2]);

			// Test index beyond array length
			removeToast(2);
			expect(getStoreValue()).toEqual([toast1, toast2]);

			// Test empty store
			toastsStore.set([]);
			removeToast(0);
			expect(getStoreValue()).toEqual([]);
		});
	});
});
