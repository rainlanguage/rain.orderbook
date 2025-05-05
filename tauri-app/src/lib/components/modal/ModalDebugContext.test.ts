import { expect, describe, it } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import ModalDebugContext from './ModalDebugContext.svelte';

describe('ModalDebugContext', () => {
  it('test no deployments network', () => {
    const modal = render(ModalDebugContext, {
      props: {
        open: true,
        networks: undefined,
        blockNumbers: {},
      },
    });

    expect(modal.baseElement).toHaveTextContent('Found no deployment');
  });

  it('test with networks without block numbers', () => {
    const networks: Record<number, string> = {
      1: 'abcd',
      2: 'efgh',
    };
    render(ModalDebugContext, {
      props: {
        open: true,
        networks,
        blockNumbers: {},
      },
    });

    // all passed on networks should be present
    for (const chainId in networks) {
      const name = screen.getByTestId(`network-name-${chainId}`);
      expect(name).toHaveTextContent(networks[Number(chainId)]);

      // empty inputs
      const block = screen.getByTestId(`chain-block-${chainId}`);
      expect(block).toHaveValue(null);

      // should have placeholder
      const input = screen.getByTestId(`chain-block-${chainId}`);
      expect(input.getAttribute('placeholder')).toEqual('Enter Block Height');
    }
  });

  it('test with networks with block numbers', () => {
    const networks: Record<number, string> = {
      1: 'abcd',
      2: 'efgh',
    };
    const blockNumbers: Record<number, number> = {
      1: 12345,
      2: 67890,
    };
    render(ModalDebugContext, {
      props: {
        open: true,
        networks,
        blockNumbers,
      },
    });

    // all passed on networks should be present
    for (const chainId in networks) {
      const name = screen.getByTestId(`network-name-${chainId}`);
      expect(name).toHaveTextContent(networks[Number(chainId)]);

      // should have specified block numbers
      const block = screen.getByTestId(`chain-block-${chainId}`);
      expect(block).toHaveValue(blockNumbers[Number(chainId)]);
    }
  });
});
