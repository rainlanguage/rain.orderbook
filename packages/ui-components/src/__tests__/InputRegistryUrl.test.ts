import { render, screen, fireEvent } from '@testing-library/svelte';
import { vi } from 'vitest';
import InputRegistryUrl from '../lib/components/input/InputRegistryUrl.svelte';
import userEvent from '@testing-library/user-event';

describe('InputRegistryUrl', () => {
	const mockPushState = vi.fn();
	const mockReload = vi.fn();
	const mockLocalStorageSetItem = vi.fn();

	beforeEach(() => {
		vi.stubGlobal('localStorage', {
			setItem: mockLocalStorageSetItem
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
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('should render input and button', () => {
		render(InputRegistryUrl, { props: { newRegistryUrl: '' } });

		const input = screen.getByPlaceholderText('Enter URL to raw strategy registry file');
		const button = screen.getByText('Load Registry URL');

		expect(input).toBeInTheDocument();
		expect(button).toBeInTheDocument();
	});

	it('should bind input value to newRegistryUrl prop', async () => {
		const screen = render(InputRegistryUrl, { props: { newRegistryUrl: '' } });

		const input = screen.getByPlaceholderText('Enter URL to raw strategy registry file');
		const testUrl = 'https://example.com/registry.json';

		await userEvent.type(input, testUrl);

		expect(input).toHaveValue(testUrl);
	});

	it('should handle registry URL loading when button is clicked', async () => {
		const testUrl = 'https://example.com/registry.json';
		render(InputRegistryUrl, { props: { newRegistryUrl: testUrl } });

		const button = screen.getByText('Load Registry URL');
		await fireEvent.click(button);

		// Verify URL update
		expect(mockPushState).toHaveBeenCalledWith({}, '', '/test-path?registry=' + testUrl);

		// Verify page reload
		expect(mockReload).toHaveBeenCalled();

		// Verify localStorage update
		expect(mockLocalStorageSetItem).toHaveBeenCalledWith('registry', testUrl);
	});

	it('should handle empty URL', async () => {
		render(InputRegistryUrl, { props: { newRegistryUrl: '' } });

		const button = screen.getByText('Load Registry URL');
		await fireEvent.click(button);

		expect(mockPushState).toHaveBeenCalledWith({}, '', '/test-path?registry=');
		expect(mockReload).toHaveBeenCalled();
		expect(mockLocalStorageSetItem).toHaveBeenCalledWith('registry', '');
	});
});
