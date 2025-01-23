import {render, cleanup, screen, fireEvent} from '@testing-library/svelte';
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
		IconExternalLink: MockComponent
	};
});

vi.mock('svelte/store', () => {
	return {
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
	it('renders sidebar when bars button is clicked', () => {
		// Mock small screen width
		mockWindowSize(500);
		const mockColorTheme = writable('light');
		const mockPage = {
			url: { pathname: '/' }
		};
		render(Sidebar, { colorTheme: mockColorTheme, page: mockPage });

		const barsButton = screen.getByTestId('sidebar-bars');
		fireEvent.click(barsButton);
		expect(screen.getByTestId('sidebar')).toBeInTheDocument();
	});
});
