import { render, cleanup, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, afterEach } from 'vitest';
import Sidebar from '../lib/components/Sidebar.svelte';
import { writable } from 'svelte/store';

vi.mock('@rainlanguage/ui-components', async () => {
	const MockComponent = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return {
		ButtonDarkMode: MockComponent,
		logoLight: 'mock-logo-light.svg',
		logoDark: 'mock-logo-dark.svg',
		IconTelegram: MockComponent,
		IconExternalLink: MockComponent,
		WalletConnect: MockComponent,
		TransactionList: MockComponent
	};
});

vi.mock('svelte/store', async (importOriginal) => {
	return {
		...((await importOriginal()) as object),
		writable: () => ({
			subscribe: () => {
				return () => {};
			},
			set: vi.fn()
		})
	};
});

const mockWindowSize = (width: number) => {
	Object.defineProperty(window, 'innerWidth', { writable: true, configurable: true, value: width });
	window.dispatchEvent(new Event('resize'));
};

describe('Sidebar', () => {
	afterEach(() => {
		cleanup();
	});

	it('renders correctly with colorTheme store', async () => {
		const mockColorTheme = writable('light');
		const mockPage = {
			url: { pathname: '/' }
		};
		const { container } = render(Sidebar, {
			props: {
				colorTheme: mockColorTheme,
				page: mockPage
			}
		});

		expect(container).toBeTruthy();
	});
	it('renders menu bars button when screen width is small', () => {
		// Mock small screen width
		mockWindowSize(500);
		const mockColorTheme = writable('light');
		const mockPage = {
			url: { pathname: '/' }
		};
		render(Sidebar, { colorTheme: mockColorTheme, page: mockPage });

		const barsButton = screen.getByTestId('sidebar-bars');
		expect(barsButton).toBeInTheDocument();
	});
	it('shows sidebar on wide screen', () => {
		mockWindowSize(1025);
		const mockColorTheme = writable('light');
		const mockPage = {
			url: { pathname: '/' }
		};
		render(Sidebar, { colorTheme: mockColorTheme, page: mockPage });

		expect(screen.getByTestId('sidebar')).toBeInTheDocument();
	});
	it('renders sidebar when bars button is clicked', async () => {
		// Mock small screen width
		mockWindowSize(500);
		const mockColorTheme = writable('light');
		const mockPage = {
			url: { pathname: '/' }
		};
		render(Sidebar, { colorTheme: mockColorTheme, page: mockPage });

		const barsButton = screen.getByTestId('sidebar-bars');
		fireEvent.click(barsButton);

		await waitFor(() => {
			const sidebar = screen.getByTestId('sidebar');
			expect(sidebar.hidden).toBe(false);
		});
	});
	it('hides sidebar when close button is clicked', async () => {
		mockWindowSize(500);
		const mockColorTheme = writable('light');
		const mockPage = {
			url: { pathname: '/' }
		};
		render(Sidebar, { colorTheme: mockColorTheme, page: mockPage });

		const barsButton = screen.getByTestId('sidebar-bars');
		await fireEvent.click(barsButton);
		await waitFor(() => {
			expect(screen.getByTestId('sidebar')).toBeInTheDocument();
		});
		const closeButton = screen.getByTestId('close-button');
		await fireEvent.click(closeButton);
		await waitFor(() => {
			const sidebar = screen.getByTestId('sidebar');
			expect(sidebar.hidden).toBe(true);
		});
	});
});
