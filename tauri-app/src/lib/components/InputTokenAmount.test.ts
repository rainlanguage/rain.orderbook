import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import InputTokenAmount from './InputTokenAmount.svelte';

describe('InputTokenAmount', () => {
  it('should handle input with 18 decimals', async () => {
    const { getByRole, component } = render(InputTokenAmount, {
      props: { decimals: 18, value: 0n },
    });
    const input = getByRole('textbox');

    await fireEvent.input(input, { target: { value: '1.5' } });
    expect(component.$$.ctx[component.$$.props.value]).toBe(1500000000000000000n);

    await fireEvent.input(input, { target: { value: '0.000000000000000001' } });
    expect(component.$$.ctx[component.$$.props.value]).toBe(1n);

    await fireEvent.input(input, { target: { value: '1000000' } });
    expect(component.$$.ctx[component.$$.props.value]).toBe(1000000000000000000000000n);
  });

  it('should handle input with 6 decimals', async () => {
    const { getByRole, component } = render(InputTokenAmount, {
      props: { decimals: 6, value: 0n },
    });
    const input = getByRole('textbox');

    await fireEvent.input(input, { target: { value: '1.5' } });
    expect(component.$$.ctx[component.$$.props.value]).toBe(1500000n);

    await fireEvent.input(input, { target: { value: '0.000001' } });
    expect(component.$$.ctx[component.$$.props.value]).toBe(1n);

    await fireEvent.input(input, { target: { value: '1000000' } });
    expect(component.$$.ctx[component.$$.props.value]).toBe(1000000000000n);
  });

  it('should handle input with 0 decimals', async () => {
    const { getByRole, component } = render(InputTokenAmount, {
      props: { decimals: 0, value: 0n },
    });
    const input = getByRole('textbox');

    await fireEvent.input(input, { target: { value: '1' } });
    expect(component.$$.ctx[component.$$.props.value]).toBe(1n);

    await fireEvent.input(input, { target: { value: '1000000' } });
    expect(component.$$.ctx[component.$$.props.value]).toBe(1000000n);
  });

  it('should handle empty input', async () => {
    const { getByRole, component } = render(InputTokenAmount, {
      props: { decimals: 18, value: 0n },
    });
    const input = getByRole('textbox');

    await fireEvent.input(input, { target: { value: '' } });
    expect(component.$$.ctx[component.$$.props.value]).toBe(0n);
  });

  it('should handle invalid input', async () => {
    const { getByRole, component } = render(InputTokenAmount, {
      props: { decimals: 18, value: 0n },
    });
    const input = getByRole('textbox');

    await fireEvent.input(input, { target: { value: 'abc' } });
    expect(component.$$.ctx[component.$$.props.value]).toBe(0n);
  });

  it('should handle maxValue prop', async () => {
    const { getByText, component } = render(InputTokenAmount, {
      props: { decimals: 18, maxValue: 1000000000000000000n, value: 0n },
    });
    const maxButton = getByText('MAX');

    await fireEvent.click(maxButton);
    expect(component.$$.ctx[component.$$.props.value]).toBe(1000000000000000000n);
  });
});
