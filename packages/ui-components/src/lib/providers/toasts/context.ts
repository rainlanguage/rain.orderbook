import { getContext, setContext } from 'svelte';
import { writable, type Writable } from 'svelte/store';

export const TOASTS_KEY = 'toasts_key';

export function createToastsContext() {
	const toasts = writable<string[]>([]);
	setContext(TOASTS_KEY, toasts);
	return toasts;
}

/**
 * Retrieves the toasts directly from Svelte's context
 */
export function getToastsContext(): Writable<string[]> {
	const toasts = getContext<Writable<string[]>>(TOASTS_KEY);
	if (!toasts) {
		throw new Error(
			'No toasts context found. Did you forget to wrap your component with ToastProvider?'
		);
	}
	return toasts;
}

/**
 * Sets the toasts in Svelte's context
 */
export const setToastsContext = (toasts: string[]) => {
	setContext(TOASTS_KEY, toasts);
};
