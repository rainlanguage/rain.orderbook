import { getToastsContext } from './context';
import { get } from 'svelte/store';
import type { ToastProps } from '$lib/types/toast';
import { v4 as uuidv4 } from 'uuid';

/**
 * Hook for managing toast notifications in the application.
 * Provides functionality to add, remove, and access toast notifications.
 *
 * @returns An object containing the toast store and methods to manipulate toasts
 */
export function useToasts() {
	const toasts = getToastsContext();

	/**
	 * Removes a toast notification by its index
	 *
	 * @param index - The index of the toast to remove
	 */
	const removeToast = (index: number) => {
		toasts.update((toasts) => {
			if (index < 0 || index >= toasts.length) {
				return toasts;
			}
			return toasts.filter((_, i) => i !== index);
		});
	};

	/**
	 * Adds a new toast notification and automatically removes it after 3 seconds
	 *
	 * @param toast - The toast properties (message and type)
	 */
	const addToast = (toast: Omit<ToastProps, 'id'>) => {
		const newToast: ToastProps = { ...toast, id: uuidv4() };

		toasts.update((toasts) => {
			const updatedToasts = [...toasts, newToast];
			return updatedToasts;
		});
	};

	return {
		toasts,
		addToast,
		removeToast
	};
}
