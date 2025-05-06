import { getToastsContext } from './context';
import { get } from 'svelte/store';
import type { ToastProps } from '$lib/types/toast';

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
		toasts.update((toasts) => toasts.filter((_, i) => i !== index));
	};

	/**
	 * Adds a new toast notification and automatically removes it after 5 seconds
	 * 
	 * @param toast - The toast properties (message and type)
	 */
	const addToast = (toast: ToastProps) => {
		const newToast = toast;

		let addedToastIndex = -1;
		toasts.update((toasts) => {
			const updatedToasts = [...toasts, newToast];
			addedToastIndex = updatedToasts.findIndex((t) => t === newToast);
			return updatedToasts;
		});

		if (addedToastIndex > -1) {
			setTimeout(() => {
				const currentToasts = get(toasts);
				const currentIndex = currentToasts.findIndex((t) => t === newToast);
				if (currentIndex > -1) {
					removeToast(currentIndex);
				}
			}, 3000);
		}
	};

	return {
		toasts,
		addToast,
		removeToast
	};
}
