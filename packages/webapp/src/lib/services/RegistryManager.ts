import { REGISTRY_URL } from '$lib/constants';

export default class RegistryManager {
	private static STORAGE_KEY = 'registry';

	static getFromStorage(): string | null {
		try {
			return localStorage.getItem(this.STORAGE_KEY);
		} catch (error) {
			throw new Error('Failed to access localStorage: ' + (error instanceof Error ? error.message : String(error)));
		}
	}

	static setToStorage(value: string): void {
		try {
			localStorage.setItem(this.STORAGE_KEY, value);
		} catch (error) {
			throw new Error('Failed to save to localStorage: ' + (error instanceof Error ? error.message : String(error)));
		}
	}

	static clearFromStorage(): void {
		try {
			localStorage.removeItem(this.STORAGE_KEY);
			this.updateUrlParam(null);
		} catch (error) {
			throw new Error('Failed to clear registry from localStorage: ' + (error instanceof Error ? error.message : String(error)));
		}
	}

	static updateUrlParam(value: string | null): void {
		try {
			const url = new URL(window.location.href);
			if (value) {
				url.searchParams.set('registry', value);
			} else {
				url.searchParams.delete('registry');
			}
			window.history.pushState({}, '', url.toString());
		} catch (error) {
			throw new Error('Failed to update URL parameter: ' + (error instanceof Error ? error.message : String(error)));
		}
	}

	static isCustomRegistry(value: string | null): boolean {
		return !!value && value !== REGISTRY_URL;
	}

	static hasRegistryParam(): boolean {
		try {
			return new URL(window.location.href).searchParams.has('registry');
		} catch (error) {
			throw new Error('Failed to check if registry parameter exists: ' + (error instanceof Error ? error.message : String(error)));
		}
	}

	static getRegistryParam(): string | null {
		try {
			return new URL(window.location.href).searchParams.get('registry');
		} catch (error) {
			throw new Error('Failed to get registry parameter: ' + (error instanceof Error ? error.message : String(error)));
		}
	}
}