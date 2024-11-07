import { render, screen } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import CardProperty from './CardProperty.svelte';

describe('Page Component', () => {
	it('should render the card component', async () => {
		render(CardProperty);
		expect(screen.getByTestId("card-property-key")).toBeInTheDocument();
	});
});
