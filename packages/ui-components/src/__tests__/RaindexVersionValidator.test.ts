import { describe, test, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import RaindexVersionValidator from '$lib/components/RaindexVersionValidator.svelte';

describe('RaindexVersionValidator Component', () => {
	test('should show error message if error is a string that starts with "Raindex version"', () => {
		const error = 'Raindex version error';
		const comp = render(RaindexVersionValidator, { props: { error } });

		const alert = comp.getByRole('alert');
		expect(alert).toContainHTML('<span>Raindex version error</span>');
		expect(alert).toHaveTextContent(
			`This order may not be compatible with this version of Raindex. `
		);
	});

	test('should not show error message if error is not a string', () => {
		const error = 42;
		const comp = render(RaindexVersionValidator, { props: { error } });

		expect(() => comp.getByRole('alert')).toThrow();
	});

	test('should not show error message if error does not start with "Raindex version"', () => {
		const error = 'Some other error';
		const comp = render(RaindexVersionValidator, { props: { error } });

		expect(() => comp.getByRole('alert')).toThrow();
	});
});
