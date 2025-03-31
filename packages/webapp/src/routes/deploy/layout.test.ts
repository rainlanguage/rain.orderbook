import { describe, expect, it, beforeAll, afterAll, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { vi } from 'vitest';
import Layout from './+layout.svelte';

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
});
