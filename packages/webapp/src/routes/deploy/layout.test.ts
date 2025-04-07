import { describe, expect, it, beforeAll, afterAll, beforeEach } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import { vi } from 'vitest';
import Layout from './+layout.svelte';
import RegistryManager from '$lib/services/registryManager';
import { REGISTRY_URL } from '$lib/constants';

const { mockPageStore } = await vi.hoisted(() => import('$lib/__mocks__/stores'));

vi.mock('$app/stores', () => {
	return { page: mockPageStore };
});

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

describe('Layout Component', () => {
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
		mockPageStore.reset(); // Reset the mock store to initial state
		vi.unstubAllGlobals();
	});

	it('should show advanced mode toggle on deploy page', () => {
		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/deploy'
			} as unknown as URL
		});

		render(Layout);

		expect(screen.getByText('Advanced mode')).toBeInTheDocument();
	});

	it('should not show advanced mode toggle on non-deploy pages', () => {
		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/other-page'
			} as unknown as URL
		});

		render(Layout);

		expect(screen.queryByText('Advanced mode')).toBeNull();
	});

	it('should show custom registry warning when using non-default registry', () => {
		localStorageMock.setItem('registry', 'https://custom-registry.com');

		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/deploy'
			} as unknown as URL
		});

		render(Layout);

		expect(localStorageMock.getItem).toHaveBeenCalledWith('registry');

		expect(screen.getByTestId('custom-registry-warning')).toBeTruthy();
	});

	it('should display InputRegistryUrl when advanced mode is on', () => {
		localStorageMock.setItem('registry', 'https://custom-registry.com');

		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/deploy'
			} as unknown as URL
		});

		render(Layout);

		expect(screen.getByTestId('registry-input')).toBeTruthy();
	});

	it('should not display InputRegistryUrl when advanced mode is off', () => {
		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/deploy'
			} as unknown as URL
		});

		render(Layout);

		expect(screen.queryByTestId('registry-input')).toBeNull();
	});
	it('should update URL with registry query parameter when registry is in localStorage', () => {
		// Mock full window.location object
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

		// Mock history.pushState method
		const historyMock = {
			...window.history,
			pushState: vi.fn()
		};

		// Apply mocks
		vi.stubGlobal('location', locationMock);
		vi.stubGlobal('history', historyMock);

		// Set up test
		localStorageMock.setItem('registry', 'https://custom-registry.com');

		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/deploy'
			} as unknown as URL
		});

		render(Layout);

		// Check if pushState was called correctly
		expect(historyMock.pushState).toHaveBeenCalledWith(
			{},
			'',
			'http://localhost/deploy?registry=https%3A%2F%2Fcustom-registry.com'
		);
	});

	it('should not update URL when no registry is in localStorage', () => {
		const pushStateSpy = vi.spyOn(window.history, 'pushState');

		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/deploy'
			} as unknown as URL
		});

		render(Layout);

		expect(pushStateSpy).not.toHaveBeenCalled();
	});
	it('should clear registry from localStorage when "Use Default" is clicked', async () => {
		localStorageMock.setItem('registry', 'https://custom-registry.com');

		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/deploy'
			} as unknown as URL
		});

		const { getByText } = render(Layout);

		const useDefaultButton = getByText('Use default.');
		await fireEvent.click(useDefaultButton);

		expect(localStorageMock.removeItem).toHaveBeenCalledWith('registry');
	});
	it('should not update URL when registry parameter is already present', () => {
		vi.stubGlobal('location', {
			pathname: '/deploy',
			search: '?registry=https://custom-registry.com'
		});

		localStorageMock.setItem('registry', 'https://custom-registry.com');

		const pushStateSpy = vi.spyOn(window.history, 'pushState');

		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/deploy'
			} as unknown as URL
		});

		render(Layout);

		expect(pushStateSpy).not.toHaveBeenCalled();
	});
});
