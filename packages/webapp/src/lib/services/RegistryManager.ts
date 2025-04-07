import { REGISTRY_URL } from '$lib/constants';

export default class RegistryManager {
	private static STORAGE_KEY = 'registry';

	static getFromStorage(): string | null {
		return localStorage.getItem(this.STORAGE_KEY);
	}

	static setToStorage(value: string): void {
		localStorage.setItem(this.STORAGE_KEY, value);
	}

	static clearFromStorage(): void {
		localStorage.removeItem(this.STORAGE_KEY);
		this.updateUrlParam(null);
	}

	static updateUrlParam(value: string | null): void {
		const url = new URL(window.location.href);
		if (value) {
			url.searchParams.set('registry', value);
		} else {
			url.searchParams.delete('registry');
		}
		window.history.pushState({}, '', url.toString());
	}

	static isCustomRegistry(value: string | null): boolean {
		return !!value && value !== REGISTRY_URL;
	}

	static hasRegistryParam(): boolean {
		return new URL(window.location.href).searchParams.has('registry');
	}

	static getRegistryParam(): string | null {
		return new URL(window.location.href).searchParams.get('registry');
	}
}
