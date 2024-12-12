import { describe, test, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import EditableSpan from '../lib/components/EditableSpan.svelte';

describe('Editable Span', () => {
	test('should show the correct value', async () => {
		const screen = render(EditableSpan, {
			displayValue: '123'
		});

		expect(screen.getByText('Block:')).toBeInTheDocument();

		// test that the input is not visible
		expect(screen.getByTestId('editableSpan')).toHaveClass('opacity-0');

		// test that the input is visible when clicked
		await screen.getByTestId('editableSpanWrapper').click();
		expect(screen.getByTestId('editableSpan')).not.toHaveClass('opacity-0');

		// test that the input is hidden when the enter key is pressed
		screen.getByTestId('editableSpan').click();
		screen
			.getByTestId('editableSpan')
			.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }));
	});
});
