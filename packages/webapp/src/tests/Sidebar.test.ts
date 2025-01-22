import { render, cleanup } from '@testing-library/svelte';
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
});
