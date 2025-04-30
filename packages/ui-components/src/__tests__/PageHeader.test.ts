import { render, screen, within } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import PageHeader from '../lib/components/PageHeader.svelte';

vi.mock('../lib/utils/breadcrumbs', () => ({
	generateBreadcrumbs: vi.fn()
}));

import { generateBreadcrumbs } from '../lib/utils/breadcrumbs';

describe('PageHeader.svelte', () => {
	const mockTitle = 'Test Page';
	const mockPathname = '/test/path';
	const mockCrumbs = [
		{ label: 'Test', href: '/test' },
		{ label: 'Path', href: '/test/path' }
	];

	beforeEach(() => {
		vi.clearAllMocks();
		vi.mocked(generateBreadcrumbs).mockReturnValue(mockCrumbs);
	});

	it('renders the title correctly', () => {
		render(PageHeader, { props: { title: mockTitle, pathname: mockPathname } });

		expect(screen.getByTestId('breadcrumb-page-title')).toHaveTextContent(mockTitle);
	});

	it('calls generateBreadcrumbs with the correct pathname', () => {
		render(PageHeader, { props: { title: mockTitle, pathname: mockPathname } });

		expect(generateBreadcrumbs).toHaveBeenCalledTimes(1);
		expect(generateBreadcrumbs).toHaveBeenCalledWith(mockPathname);
	});

	it('renders the generated breadcrumbs', () => {
		render(PageHeader, { props: { title: mockTitle, pathname: mockPathname } });

		const nav = screen.getByRole('navigation', { name: 'Default breadcrumb example' });

		const links = within(nav).getAllByRole('link');

		expect(links[0]).toHaveAttribute('href', '/');

		mockCrumbs.forEach((crumb) => {
			expect(screen.getByRole('link', { name: crumb.label })).toHaveAttribute('href', crumb.href);
		});
	});
});
