import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { writable } from 'svelte/store';
import { useToasts } from '$lib/providers/toasts/useToasts';
import type { ToastProps } from '$lib/types/toast';
import { getToastsContext } from '$lib/providers/toasts/context';

vi.mock('$lib/providers/toasts/context', () => ({
	getToastsContext: vi.fn()
}));

vi.mock('uuid', () => ({
	v4: vi.fn().mockReturnValue('mocked-uuid')
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
		vi.useFakeTimers();
		toastsStore = writable<ToastProps[]>([]);
		vi.mocked(getToastsContext).mockReturnValue(toastsStore);
	});

	afterEach(() => {
		vi.useRealTimers();
		vi.clearAllMocks();
	});

	it('should return the toasts store and functions', () => {
		const result = useToasts();

		expect(result.toasts).toBe(toastsStore);
		expect(typeof result.addToast).toBe('function');
		expect(typeof result.removeToast).toBe('function');
	});

	describe('addToast', () => {
		it('should add a toast and schedule its removal', () => {
			const { addToast } = useToasts();
			const testToast: ToastProps = {
				message: 'Test Toast',
				type: 'info',
				color: 'green',
				id: 'mocked-uuid'
			};

			addToast(testToast);
			expect(getStoreValue()).toEqual([testToast]);

			vi.advanceTimersByTime(3000);
			expect(getStoreValue()).toEqual([]);
		});

		it('should only remove the correct toast after state changes', () => {
			const { addToast, removeToast } = useToasts();
			const toast1: ToastProps = {
				message: 'Toast 1',
				type: 'info',
				color: 'green',
				id: 'mocked-uuid'
			};
			const toast2: ToastProps = {
				message: 'Toast 2',
				type: 'info',
				color: 'green',
				id: 'mocked-uuid'
			};

			addToast(toast1);
			addToast(toast2);
			removeToast(0);

			expect(getStoreValue()).toEqual([toast2]);

			vi.advanceTimersByTime(3000);
			expect(getStoreValue()).toEqual([]);
		});

		it('should add a toast with missing properties', () => {
			const { addToast } = useToasts();
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const testToast = { message: 'Test Toast Only Message', id: 'mocked-uuid' } as any;

			addToast(testToast);
			expect(getStoreValue()).toEqual([testToast]);

			vi.advanceTimersByTime(3000);
			expect(getStoreValue()).toEqual([]);
		});

		it('should add multiple toasts in rapid succession', () => {
			const { addToast } = useToasts();
			const toast1: ToastProps = {
				message: 'Toast 1',
				type: 'info',
				color: 'green',
				id: 'mocked-uuid'
			};
			const toast2: ToastProps = {
				message: 'Toast 2',
				type: 'success',
				color: 'blue',
				id: 'mocked-uuid'
			};
			const toast3: ToastProps = {
				message: 'Toast 3',
				type: 'error',
				color: 'red',
				id: 'mocked-uuid'
			};

			addToast(toast1);
			addToast(toast2);
			addToast(toast3);

			expect(getStoreValue()).toEqual([toast1, toast2, toast3]);

			vi.advanceTimersByTime(3000);

			expect(getStoreValue()).toEqual([]);
		});
	});

	describe('removeToast', () => {
		it('should remove a toast at the specified index', () => {
			const { removeToast } = useToasts();
			const initialToasts: ToastProps[] = [
				{ message: 'Toast 0', type: 'info', color: 'green', id: 'mocked-uuid' },
				{ message: 'Toast 1', type: 'info', color: 'green', id: 'mocked-uuid' },
				{ message: 'Toast 2', type: 'info', color: 'green', id: 'mocked-uuid' }
			];

			toastsStore.set(initialToasts);
			removeToast(1);

			expect(getStoreValue()).toEqual([
				{ message: 'Toast 0', type: 'info', color: 'green', id: 'mocked-uuid' },
				{ message: 'Toast 2', type: 'info', color: 'green', id: 'mocked-uuid' }
			]);
		});

		it('should not throw an error or change the list when removing with an out-of-bounds index', () => {
			const { removeToast } = useToasts();
			const initialToasts: ToastProps[] = [
				{ message: 'Toast 0', type: 'info', color: 'green', id: 'mocked-uuid' },
				{ message: 'Toast 1', type: 'info', color: 'green', id: 'mocked-uuid' }
			];
			toastsStore.set(initialToasts);

			removeToast(-1);
			expect(getStoreValue()).toEqual(initialToasts);

			removeToast(initialToasts.length);
			expect(getStoreValue()).toEqual(initialToasts);

			toastsStore.set([]);
			removeToast(0);
			expect(getStoreValue()).toEqual([]);
		});
	});
});
