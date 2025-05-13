import { expect, describe, it, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import ModalDebugContext from './ModalDebugContext.svelte';
import userEvent from '@testing-library/user-event';

describe('ModalDebugContext', () => {
  it('test no deployments network', () => {
    const modal = render(ModalDebugContext, {
      props: {
        open: true,
        networks: undefined,
        blockNumbers: {},
        onClose: () => {},
      },
    });

    expect(modal.baseElement).toHaveTextContent(
      "Found no deployment, please add deployments to your order's configurations to debug it",
    );
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
        onClose: () => {},
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
        onClose: () => {},
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

  it('updates blockNumbers on input', async () => {
    const networks = { 1: 'abcd' };
    const blockNumbers = { 1: 12345 };
    render(ModalDebugContext, {
      props: { open: true, networks, blockNumbers, onClose: () => {} },
    });

    const input = screen.getByTestId('chain-block-1');
    await userEvent.clear(input);
    await userEvent.type(input, '54321');
    expect(input).toHaveValue(54321);
  });

  it('calls onClose when close button is clicked', async () => {
    const onClose = vi.fn();
    const networks = { 1: 'abcd' };
    render(ModalDebugContext, {
      props: { open: true, networks, blockNumbers: {}, onClose },
    });

    // Find the close button by its accessible name
    const closeButton = screen.getByLabelText(/close modal/i);
    expect(closeButton).toBeInTheDocument();

    // Simulate a click on the close button
    await fireEvent.click(closeButton);

    // Now onClose should have been called
    expect(onClose).toHaveBeenCalled();
  });

  it('does not render modal when open is false', () => {
    const networks = { 1: 'abcd' };
    render(ModalDebugContext, {
      props: { open: false, networks, blockNumbers: {}, onClose: () => {} },
    });
    expect(screen.queryByText('Debug Block Height')).not.toBeInTheDocument();
  });
});
