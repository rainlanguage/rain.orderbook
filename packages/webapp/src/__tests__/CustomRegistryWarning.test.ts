import { render, screen, fireEvent } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';
import CustomRegistryWarning from '$lib/components/CustomRegistryWarning.svelte';

// Create a mock for localStorage
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

describe('CustomRegistryWarning Component', () => {
	beforeEach(() => {
		vi.stubGlobal('localStorage', localStorageMock);

		localStorageMock.setItem('registry', 'https://custom-registry.com');

		vi.clearAllMocks();
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('should render the warning message correctly', () => {
		render(CustomRegistryWarning);

		const warningElement = screen.getByTestId('custom-registry-warning');
		expect(warningElement).toBeInTheDocument();

		expect(screen.getByText(/You are using a/i)).toBeInTheDocument();
		expect(screen.getByText(/custom strategies registry./i)).toBeInTheDocument();

		const defaultLink = screen.getByText('Use default.');
		expect(defaultLink).toBeInTheDocument();
		expect(defaultLink.tagName.toLowerCase()).toBe('a');
		expect(defaultLink).toHaveAttribute('href', '/deploy');
		expect(defaultLink).toHaveAttribute('data-sveltekit-reload');
	});

	it('should remove registry from localStorage when "Use default" is clicked', async () => {
		render(CustomRegistryWarning);

		const defaultLink = screen.getByText('Use default.');
		await fireEvent.click(defaultLink);

		expect(localStorageMock.removeItem).toHaveBeenCalledTimes(1);
		expect(localStorageMock.removeItem).toHaveBeenCalledWith('registry');
	});
});
