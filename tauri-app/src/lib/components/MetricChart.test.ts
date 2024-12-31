import { describe, test, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import MetricChart from './MetricChart.svelte';

describe('MetricChart Component', () => {
  test('renders metric label', () => {
    const metric = {
      label: 'Test Metric',
      'unit-prefix': '$',
      'unit-suffix': ' USD',
      value: 'testValue',
      precision: 4
    };
    const data = [{ testValue: 22 }];

    const { getByText } = render(MetricChart, { props: { metric, data } });

    expect(getByText('Test Metric')).toBeInTheDocument();
  });

  test('renders formatted data with precision', () => {
    const metric = {
      label: 'Test Metric',
      'unit-prefix': '$',
      'unit-suffix': ' USD',
      value: 'testValue',
      precision: 4
    };
    const data = [{ testValue: 123.456 }];

    const { getByText } = render(MetricChart, { props: { metric, data } });

    expect(getByText('$123.5 USD')).toBeInTheDocument();
  });

  test('renders data without precision when not provided', () => {
    const metric = {
      label: 'Test Metric',
      'unit-prefix': '$',
      'unit-suffix': ' USD',
      value: 'testValue'
    };
    const data = [{ testValue: 123.456 }];

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
      precision: 2
    };
    const data = [{ testValue: 123.456 }];

    const { getByText } = render(MetricChart, { props: { metric, data } });

    expect(getByText('This is a test metric.')).toBeInTheDocument();
  });
});
