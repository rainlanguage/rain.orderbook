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
	const addToast = (toast: ToastProps) => {
		const newToast: ToastProps = { ...toast, id: uuidv4() };

		let addedToastIndex = -1;
		toasts.update((toasts) => {
			const updatedToasts = [...toasts, newToast];
			addedToastIndex = updatedToasts.length - 1;
			return updatedToasts;
		});
		const toastId = newToast.id;

		if (addedToastIndex > -1) {
			setTimeout(() => {
				const currentToasts = get(toasts);
				const currentIndex = currentToasts.findIndex((t) => t.id === toastId);
				if (currentIndex > -1) {
					removeToast(currentIndex);
				}
			}, 3000);
		}
	};

	/**
	 * Adds a standardized error toast notification
	 *
	 * @param message - The error message to display
	 */
	const errToast = (message: string) => {
		addToast({
			message,
			type: 'error',
			color: 'red'
		});
	};

	return {
		toasts,
		addToast,
		removeToast,
		errToast
	};
}
