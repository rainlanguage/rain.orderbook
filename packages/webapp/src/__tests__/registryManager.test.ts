import { describe, expect, it, beforeAll, afterAll, beforeEach } from 'vitest';
import { vi } from 'vitest';
import fc from 'fast-check';
import { test } from '@fast-check/vitest';
import RegistryManager from '$lib/services/RegistryManager';
import { REGISTRY_URL } from '$lib/constants';

const localStorageMock: Storage = (() => {
	let store: Record<string, string> = {};

	return {
		getItem: vi.fn((key: string): string | null => store[key] || null),
		setItem: vi.fn((key: string, value: string): void => {
			store[key] = value.toString();
		}),
		removeItem: vi.fn((key: string): void => {
			delete store[key];
		}),
		clear: vi.fn((): void => {
			store = {};
		}),
		key: (index: number): string => Object.keys(store)[index] || '',
		length: 0
	};
})();

let originalLocalStorage: Storage;

describe('RegistryManager', () => {
	beforeAll((): void => {
		originalLocalStorage = window.localStorage;
		Object.defineProperty(window, 'localStorage', { value: localStorageMock });
	});

	afterAll((): void => {
		Object.defineProperty(window, 'localStorage', { value: originalLocalStorage });
	});

	beforeEach((): void => {
		localStorageMock.clear();
		vi.clearAllMocks();
		vi.unstubAllGlobals();
	});

	it('should properly clear registry from both storage and URL', () => {
		const locationMock = {
			pathname: '/deploy',
			search: '?registry=https://custom-registry.com',
			href: 'http://localhost/deploy?registry=https://custom-registry.com',
			host: 'localhost',
			hostname: 'localhost',
			origin: 'http://localhost',
			protocol: 'http:',
			port: ''
		};

		const historyMock = {
			...window.history,
			pushState: vi.fn()
		};

		// Apply mocks
		vi.stubGlobal('location', locationMock);
		vi.stubGlobal('history', historyMock);

		RegistryManager.setToStorage('https://custom-registry.com');
		RegistryManager.clearFromStorage();

		expect(localStorageMock.removeItem).toHaveBeenCalledWith('registry');
		expect(historyMock.pushState).toHaveBeenCalledWith({}, '', 'http://localhost/deploy');
	});

	it('should detect custom registry correctly', () => {
		expect(RegistryManager.isCustomRegistry('https://custom-registry.com')).toBe(true);
		expect(RegistryManager.isCustomRegistry(REGISTRY_URL)).toBe(false);
		expect(RegistryManager.isCustomRegistry(null)).toBe(false);
	});

	it('should handle URL parameters correctly', () => {
		const locationMock = {
			pathname: '/deploy',
			search: '',
			href: 'http://localhost/deploy',
			host: 'localhost',
			hostname: 'localhost',
			origin: 'http://localhost',
			protocol: 'http:',
			port: ''
		};

		const historyMock = {
			...window.history,
			pushState: vi.fn()
		};

		vi.stubGlobal('location', locationMock);
		vi.stubGlobal('history', historyMock);

		RegistryManager.updateUrlParam('https://custom-registry.com');
		expect(historyMock.pushState).toHaveBeenCalledWith(
			{},
			'',
			'http://localhost/deploy?registry=https%3A%2F%2Fcustom-registry.com'
		);

		historyMock.pushState.mockReset();

		RegistryManager.updateUrlParam(null);
		expect(historyMock.pushState).toHaveBeenCalledWith({}, '', 'http://localhost/deploy');
	});
	it('should get registry value from storage', () => {
		localStorageMock.setItem('registry', 'https://custom-registry.com');
		expect(RegistryManager.getFromStorage()).toBe('https://custom-registry.com');
	});

	it('should remove registry value from storage', () => {
		RegistryManager.setToStorage('https://custom-registry.com');
		expect(RegistryManager.getFromStorage()).toBe('https://custom-registry.com');
		RegistryManager.clearFromStorage();
		expect(localStorageMock.removeItem).toHaveBeenCalledWith('registry');
		expect(RegistryManager.getFromStorage()).toBe(null);
	});

	it('should set registry value to storage', () => {
		RegistryManager.setToStorage('https://custom-registry.com');
		expect(localStorageMock.setItem).toHaveBeenCalledWith(
			'registry',
			'https://custom-registry.com'
		);
	});

	it('should correctly determine if registry parameter exists in URL', () => {
		vi.stubGlobal('location', {
			href: 'http://localhost/deploy?registry=https://custom-registry.com'
		});
		expect(RegistryManager.hasRegistryParam()).toBe(true);

		vi.stubGlobal('location', {
			href: 'http://localhost/deploy'
		});
		expect(RegistryManager.hasRegistryParam()).toBe(false);
	});

	it('should get registry parameter from URL', () => {
		vi.stubGlobal('location', {
			href: 'http://localhost/deploy?registry=https://custom-registry.com'
		});
		expect(RegistryManager.getRegistryParam()).toBe('https://custom-registry.com');

		vi.stubGlobal('location', {
			href: 'http://localhost/deploy'
		});
		expect(RegistryManager.getRegistryParam()).toBe(null);
	});

	it('should handle extremely long URLs (approaching browser limits)', () => {
		const baseUrl = 'https://example.com/registry?';
		const padding = 'x'.repeat(2020 - baseUrl.length);
		const longUrl = baseUrl + padding;

		expect(longUrl.length).toBe(2020);

		RegistryManager.setToStorage(longUrl);
		expect(RegistryManager.getFromStorage()).toBe(longUrl);

		const locationMock = {
			pathname: '/deploy',
			search: '',
			href: 'http://localhost/deploy',
			host: 'localhost',
			hostname: 'localhost',
			origin: 'http://localhost',
			protocol: 'http:',
			port: ''
		};

		const historyMock = {
			...window.history,
			pushState: vi.fn()
		};

		vi.stubGlobal('location', locationMock);
		vi.stubGlobal('history', historyMock);

		RegistryManager.updateUrlParam(longUrl);

		expect(historyMock.pushState).toHaveBeenCalled();
	});

	test.prop([fc.webUrl(), fc.string()])(
		'should correctly update URL with any valid registry URL',
		(registryUrl, pathname) => {
			const sanitizedPathname = pathname.startsWith('/') ? pathname : `/${pathname}`;

			const locationMock = {
				pathname: sanitizedPathname,
				search: '',
				href: `http://localhost${sanitizedPathname}`,
				host: 'localhost',
				hostname: 'localhost',
				origin: 'http://localhost',
				protocol: 'http:',
				port: ''
			};

			const historyMock = {
				...window.history,
				pushState: vi.fn()
			};

			vi.stubGlobal('location', locationMock);
			vi.stubGlobal('history', historyMock);

			RegistryManager.updateUrlParam(registryUrl);

			expect(historyMock.pushState).toHaveBeenCalled();

			const generatedUrl = historyMock.pushState.mock.calls[0][2];

			const url = new URL(generatedUrl);
			const extractedRegistry = url.searchParams.get('registry');

			expect(extractedRegistry).toBe(registryUrl);
		}
	);

	test.prop([fc.webUrl()])(
		'should store and retrieve any valid registry URL correctly',
		(registryUrl) => {
			RegistryManager.setToStorage(registryUrl);

			expect(localStorageMock.setItem).toHaveBeenCalledWith('registry', registryUrl);

			const retrievedValue = RegistryManager.getFromStorage();

			expect(retrievedValue).toBe(registryUrl);
		}
	);

	test.prop([fc.webUrl()])('should correctly identify custom registries', (registryUrl) => {
		const isCustom = RegistryManager.isCustomRegistry(registryUrl);

		expect(isCustom).toBe(registryUrl !== REGISTRY_URL);
	});

	test.prop([fc.webUrl(), fc.boolean()])(
		'should correctly detect registry parameters in URLs',
		(registryUrl, includeParam) => {
			const url = includeParam
				? `http://localhost/deploy?registry=${encodeURIComponent(registryUrl)}`
				: 'http://localhost/deploy';

			vi.stubGlobal('location', { href: url });

			const hasParam = RegistryManager.hasRegistryParam();

			expect(hasParam).toBe(includeParam);

			if (includeParam) {
				const retrievedParam = RegistryManager.getRegistryParam();
				expect(retrievedParam).toBe(registryUrl);
			}
		}
	);

	test.prop([
		fc.webUrl(),
		fc.record({
			otherParam1: fc.string(),
			otherParam2: fc.string()
		})
	])('should handle URLs with multiple parameters correctly', (registryUrl, otherParams) => {
		const url = `http://localhost/deploy?otherParam1=${encodeURIComponent(otherParams.otherParam1)}&registry=${encodeURIComponent(registryUrl)}&otherParam2=${encodeURIComponent(otherParams.otherParam2)}`;

		vi.stubGlobal('location', { href: url });

		const hasParam = RegistryManager.hasRegistryParam();
		expect(hasParam).toBe(true);

		const retrievedParam = RegistryManager.getRegistryParam();
		expect(retrievedParam).toBe(registryUrl);
	});
});


describe('RegistryManager Error Handling', () => {
	let originalLocalStorage: Storage;
	let originalLocation: Location;
	let originalHistory: History;

	beforeAll((): void => {
		originalLocalStorage = window.localStorage;
		originalLocation = window.location;
		originalHistory = window.history;
	});

	afterAll((): void => {
		Object.defineProperty(window, 'localStorage', { value: originalLocalStorage });
		vi.stubGlobal('location', originalLocation);
		vi.stubGlobal('history', originalHistory);
	});

	beforeEach((): void => {
		vi.clearAllMocks();
		vi.unstubAllGlobals();
	});

	it('should throw error when getFromStorage fails to access localStorage', () => {
		Object.defineProperty(window, 'localStorage', {
			value: {
				getItem: vi.fn(() => {
					throw new Error('Storage access denied');
				})
			}
		});

		expect(() => RegistryManager.getFromStorage()).toThrow('Failed to access localStorage: Storage access denied');
	});

	it('should throw error when setToStorage fails to save to localStorage', () => {
		Object.defineProperty(window, 'localStorage', {
			value: {
				setItem: vi.fn(() => {
					throw new Error('Storage quota exceeded');
				})
			}
		});

		expect(() => RegistryManager.setToStorage('https://custom-registry.com')).toThrow(
			'Failed to save to localStorage: Storage quota exceeded'
		);
	});

	it('should throw error when clearFromStorage fails to remove from localStorage', () => {
		Object.defineProperty(window, 'localStorage', {
			value: {
				removeItem: vi.fn(() => {
					throw new Error('Cannot access storage');
				})
			}
		});

		expect(() => RegistryManager.clearFromStorage()).toThrow(
			'Failed to clear registry from localStorage: Cannot access storage'
		);
	});

	it('should throw error when updateUrlParam fails to update URL', () => {
		global.URL = vi.fn(() => {
			throw new Error('Invalid URL');
		}) as any;

		expect(() => RegistryManager.updateUrlParam('https://custom-registry.com')).toThrow(
			'Failed to update URL parameter: Invalid URL'
		);
	});

	it('should throw error when hasRegistryParam fails to check URL parameters', () => {
		global.URL = vi.fn(() => {
			throw new Error('URL parsing failed');
		}) as any;

		expect(() => RegistryManager.hasRegistryParam()).toThrow(
			'Failed to check if registry parameter exists: URL parsing failed'
		);
	});

	it('should throw error when getRegistryParam fails to get URL parameters', () => {
		global.URL = vi.fn(() => {
			throw new Error('URL malformed');
		}) as any;

		expect(() => RegistryManager.getRegistryParam()).toThrow(
			'Failed to get registry parameter: URL malformed'
		);
	});

	it('should handle non-Error objects in error handling for getFromStorage', () => {
		Object.defineProperty(window, 'localStorage', {
			value: {
				getItem: vi.fn(() => {
					throw 'Access denied';
				})
			}
		});

		expect(() => RegistryManager.getFromStorage()).toThrow('Failed to access localStorage: Access denied');
	});

	it('should handle non-Error objects in error handling for setToStorage', () => {
		Object.defineProperty(window, 'localStorage', {
			value: {
				setItem: vi.fn(() => {
					throw 404;
				})
			}
		});

		expect(() => RegistryManager.setToStorage('https://custom-registry.com')).toThrow(
			'Failed to save to localStorage: 404'
		);
	});

	it('should handle history API errors in updateUrlParam', () => {
		global.URL = class {
			searchParams = {
				set: vi.fn(),
				delete: vi.fn(),
				has: vi.fn(),
				get: vi.fn()
			};
			toString() {
				return 'http://test.com';
			}
			constructor() {
				return this;
			}
		} as any;

		Object.defineProperty(window, 'history', {
			value: {
				pushState: vi.fn(() => {
					throw new Error('History API not available');
				})
			}
		});

		expect(() => RegistryManager.updateUrlParam('https://custom-registry.com')).toThrow(
			'Failed to update URL parameter: History API not available'
		);
	});

	it('should throw error when localStorage is not available for getFromStorage', () => {
		Object.defineProperty(window, 'localStorage', { value: undefined });

		expect(() => RegistryManager.getFromStorage()).toThrow(/Failed to access localStorage/);
	});

	it('should throw error when localStorage is not available for setToStorage', () => {
		Object.defineProperty(window, 'localStorage', { value: undefined });

		expect(() => RegistryManager.setToStorage('https://custom-registry.com')).toThrow(
			/Failed to save to localStorage/
		);
	});

	it('should throw error when localStorage is not available for clearFromStorage', () => {
		Object.defineProperty(window, 'localStorage', { value: undefined });

		expect(() => RegistryManager.clearFromStorage()).toThrow(/Failed to clear registry from localStorage/);
	});
});