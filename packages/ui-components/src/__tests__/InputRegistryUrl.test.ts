import { render, screen, fireEvent } from '@testing-library/svelte';
import { vi } from 'vitest';
import InputRegistryUrl from '../lib/components/input/InputRegistryUrl.svelte';
import userEvent from '@testing-library/user-event';

describe('InputRegistryUrl', () => {
	const mockLocalStorageGetItem = vi.fn();
	const mockLoadRegistryUrl = vi.fn();

	beforeEach(() => {
		vi.stubGlobal('localStorage', {
			getItem: mockLocalStorageGetItem
		});

		mockLocalStorageGetItem.mockClear();
		mockLoadRegistryUrl.mockClear();
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('should render input and button', () => {
		mockLocalStorageGetItem.mockReturnValue('');
		render(InputRegistryUrl, { props: { loadRegistryUrl: mockLoadRegistryUrl } });

		const input = screen.getByPlaceholderText('Enter URL to raw strategy registry file');
		const button = screen.getByText('Load Registry URL');

		expect(input).toBeInTheDocument();
		expect(button).toBeInTheDocument();
	});

	it('should call loadRegistryUrl prop when button is clicked', async () => {
		mockLocalStorageGetItem.mockReturnValue('');
		render(InputRegistryUrl, { props: { loadRegistryUrl: mockLoadRegistryUrl } });

		const input = screen.getByPlaceholderText('Enter URL to raw strategy registry file');
		const testUrl = 'https://example.com/registry.json';
		await userEvent.type(input, testUrl);

		const button = screen.getByText('Load Registry URL');
		await fireEvent.click(button);

		expect(mockLoadRegistryUrl).toHaveBeenCalledWith(testUrl);
	});

	it('should load initial value from localStorage', () => {
		const initialUrl = 'https://example.com/registry.json';
		mockLocalStorageGetItem.mockReturnValue(initialUrl);

		render(InputRegistryUrl, { props: { loadRegistryUrl: mockLoadRegistryUrl } });

		const input = screen.getByPlaceholderText('Enter URL to raw strategy registry file');
		expect(input).toHaveValue(initialUrl);
		expect(mockLocalStorageGetItem).toHaveBeenCalledWith('registry');
	});
});
