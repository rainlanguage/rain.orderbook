import { render, screen, fireEvent } from '@testing-library/svelte';
import { vi } from 'vitest';
import InputRegistryUrl from '../lib/components/input/InputRegistryUrl.svelte';
import userEvent from '@testing-library/user-event';

describe('InputRegistryUrl', () => {
	const mockPushState = vi.fn();
	const mockReload = vi.fn();
	const mockLocalStorageSetItem = vi.fn();
	const mockLocalStorageGetItem = vi.fn();

	beforeEach(() => {
		vi.stubGlobal('localStorage', {
			setItem: mockLocalStorageSetItem,
			getItem: mockLocalStorageGetItem
		});

		Object.defineProperty(window, 'location', {
			value: {
				pathname: '/test-path',
				reload: mockReload
			},
			writable: true
		});

		window.history.pushState = mockPushState;

		mockPushState.mockClear();
		mockReload.mockClear();
		mockLocalStorageSetItem.mockClear();
		mockLocalStorageGetItem.mockClear();
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('should render input and button', () => {
		mockLocalStorageGetItem.mockReturnValue('');
		render(InputRegistryUrl);

		const input = screen.getByPlaceholderText('Enter URL to raw strategy registry file');
		const button = screen.getByText('Load Registry URL');

		expect(input).toBeInTheDocument();
		expect(button).toBeInTheDocument();
	});

	it('should bind input value to newRegistryUrl', async () => {
		mockLocalStorageGetItem.mockReturnValue('');
		render(InputRegistryUrl);

		const input = screen.getByPlaceholderText('Enter URL to raw strategy registry file');
		const testUrl = 'https://example.com/registry.json';

		await userEvent.type(input, testUrl);

		expect(input).toHaveValue(testUrl);
	});

	it('should handle registry URL loading when button is clicked', async () => {
		mockLocalStorageGetItem.mockReturnValue('');
		render(InputRegistryUrl);

		const input = screen.getByPlaceholderText('Enter URL to raw strategy registry file');
		const testUrl = 'https://example.com/registry.json';
		await userEvent.type(input, testUrl);

		const button = screen.getByText('Load Registry URL');
		await fireEvent.click(button);

		expect(mockPushState).toHaveBeenCalledWith({}, '', '/test-path?registry=' + testUrl);
		expect(mockReload).toHaveBeenCalled();
		expect(mockLocalStorageSetItem).toHaveBeenCalledWith('registry', testUrl);
	});

	it('should handle empty URL', async () => {
		mockLocalStorageGetItem.mockReturnValue('');
		render(InputRegistryUrl);

		const button = screen.getByText('Load Registry URL');
		await fireEvent.click(button);

		expect(mockPushState).toHaveBeenCalledWith({}, '', '/test-path?registry=');
		expect(mockReload).toHaveBeenCalled();
		expect(mockLocalStorageSetItem).toHaveBeenCalledWith('registry', '');
	});

	it('should load initial value from localStorage', () => {
		const initialUrl = 'https://example.com/registry.json';
		mockLocalStorageGetItem.mockReturnValue(initialUrl);

		render(InputRegistryUrl);

		const input = screen.getByPlaceholderText('Enter URL to raw strategy registry file');
		expect(input).toHaveValue(initialUrl);
		expect(mockLocalStorageGetItem).toHaveBeenCalledWith('registry');
	});
});
