import { describe, test, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import MetricChart from './MetricChart.svelte';
import { Float } from '@rainlanguage/orderbook';

describe('MetricChart Component', () => {
  test('renders metric label', () => {
    const metric = {
      label: 'Test Metric',
      'unit-prefix': '$',
      'unit-suffix': ' USD',
      value: 'testValue',
      precision: 4,
    };
    const data = [{ testValue: buildPlotData('22') }];

    const { getByText } = render(MetricChart, { props: { metric, data } });

    expect(getByText('Test Metric')).toBeInTheDocument();
  });

  test('renders formatted data with precision', () => {
    const metric = {
      label: 'Test Metric',
      'unit-prefix': '$',
      'unit-suffix': ' USD',
      value: 'testValue',
      precision: 4,
    };
    const data = [{ testValue: buildPlotData('123.456') }];

    const { getByText } = render(MetricChart, { props: { metric, data } });

    expect(getByText('$123.5 USD')).toBeInTheDocument();
  });

  test('renders data without precision when not provided', () => {
    const metric = {
      label: 'Test Metric',
      'unit-prefix': '$',
      'unit-suffix': ' USD',
      value: 'testValue',
    };
    const data = [{ testValue: buildPlotData('123.456') }];

    const { getByText } = render(MetricChart, { props: { metric, data } });

    expect(getByText('$123.456 USD')).toBeInTheDocument();
  });

  test('renders description if provided', () => {
    const metric = {
      label: 'Test Metric',
      description: 'This is a test metric.',
      'unit-prefix': '$',
      'unit-suffix': ' USD',
      value: 'testValue',
      precision: 2,
    };
    const data = [{ testValue: buildPlotData('123.456') }];

    const { getByText } = render(MetricChart, { props: { metric, data } });

    expect(getByText('This is a test metric.')).toBeInTheDocument();
  });
});

const buildPlotData = (value: string) => {
  const floatResult = Float.parse(value);
  if (floatResult.error || !floatResult.value) {
    throw new Error(floatResult.error?.readableMsg ?? floatResult.error?.msg ?? 'parse error');
  }
  const formattedResult = floatResult.value.format();
  if (formattedResult.error || !formattedResult.value) {
    throw new Error(
      formattedResult.error?.readableMsg ?? formattedResult.error?.msg ?? 'format error',
    );
  }

  return {
    float: floatResult.value,
    formatted: formattedResult.value as string,
    value: Number(formattedResult.value),
  };
};
