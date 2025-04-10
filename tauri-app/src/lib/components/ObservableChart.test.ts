import { describe, test, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import ObservableChart from './ObservableChart.svelte';
import type { PlotCfg } from '@rainlanguage/orderbook';

const plot: PlotCfg = {
  title: 'some title',
  subtitle: 'some subtitle',
  marks: [{ type: 'line', options: { x: '0.0', y: '0.0' } }],
};

describe('ObservableChart Component', () => {
  test('should have correct title/subtitle', () => {
    const comp = render(ObservableChart, { props: { plot, data: [] } });
    const chart = comp.getAllByTestId('chart')[0];

    expect(chart).toContainHTML(`<h2>${plot.title}</h2>`);
    expect(chart).toContainHTML(`<h3>${plot.subtitle}</h3>`);
  });
});
