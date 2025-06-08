import { describe, test, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import SpecVersionValidator from './SpecVersionValidator.svelte';

describe('SpecVersionValidator Component', () => {
  test('should show error message if error is a string that starts with "Spec version"', () => {
    const error = 'Spec version error';
    const comp = render(SpecVersionValidator, { props: { error } });

    const alert = comp.getByRole('alert');
    expect(alert).toContainHTML('<span>Spec version error</span>');
    expect(alert).toHaveTextContent(
      `This order may not be compatible with this version of Raindex. `,
    );
  });

  test('should not show error message if error is not a string', () => {
    const error = 42;
    const comp = render(SpecVersionValidator, { props: { error } });

    expect(() => comp.getByRole('alert')).toThrow();
  });

  test('should not show error message if error does not start with "Spec version"', () => {
    const error = 'Some other error';
    const comp = render(SpecVersionValidator, { props: { error } });

    expect(() => comp.getByRole('alert')).toThrow();
  });
});
