import { render, screen } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import Page from './+page.svelte';

describe('Page Component', () => {
	it('should load the page', async () => {
		render(Page);
		expect(screen.getByTestId('page-container')).toBeInTheDocument();
	});
});
