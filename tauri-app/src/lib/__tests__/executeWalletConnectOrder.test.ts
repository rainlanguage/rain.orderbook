import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
  executeWalletConnectOrder,
  type ExecuteOrderDependencies,
} from '../services/executeWalletConnectOrder';
import type { DeploymentCfg } from '@rainlanguage/orderbook';

// Mocks
const mockOrderAddCalldataFn = vi.fn();
const mockEthersExecuteFn = vi.fn();
const mockReportErrorToSentryFn = vi.fn();
const mockFormatEthersTransactionErrorFn = vi.fn();
const mockSuccessToastFn = vi.fn();
const mockErrorToastFn = vi.fn();

const mockDependencies: ExecuteOrderDependencies = {
  orderAddCalldataFn: mockOrderAddCalldataFn,
  ethersExecuteFn: mockEthersExecuteFn,
  reportErrorToSentryFn: mockReportErrorToSentryFn,
  formatEthersTransactionErrorFn: mockFormatEthersTransactionErrorFn,
  successToastFn: mockSuccessToastFn,
  errorToastFn: mockErrorToastFn,
};

const mockDotrainText = 'some dotrain text';
const mockDeployment: DeploymentCfg = {
  order: {
    orderbook: { address: '0xOrderbookAddress' },
  },
} as DeploymentCfg;
const mockCalldata = new Uint8Array([1, 2, 3]);
const mockTxResponse = { wait: vi.fn().mockResolvedValue('txReceipt') };

describe('executeWalletConnectOrder', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should throw error if orderbook address is missing', async () => {
    const depWithoutAddr: DeploymentCfg = {
      order: { orderbook: {} }, // no address
    } as DeploymentCfg;
    await expect(
      executeWalletConnectOrder(mockDotrainText, depWithoutAddr, mockDependencies),
    ).rejects.toThrow('No orderbook associated with scenario');
  });

  it('should execute full flow successfully', async () => {
    mockOrderAddCalldataFn.mockResolvedValue(mockCalldata);
    mockEthersExecuteFn.mockResolvedValue(mockTxResponse);

    await executeWalletConnectOrder(mockDotrainText, mockDeployment, mockDependencies);

    expect(mockOrderAddCalldataFn).toHaveBeenCalledWith(mockDotrainText, mockDeployment);
    expect(mockEthersExecuteFn).toHaveBeenCalledWith(
      mockCalldata,
      mockDeployment.order.orderbook?.address,
    );
    expect(mockSuccessToastFn).toHaveBeenCalledWith('Transaction sent successfully!');
    expect(mockTxResponse.wait).toHaveBeenCalledWith(1);
    expect(mockReportErrorToSentryFn).not.toHaveBeenCalled();
    expect(mockErrorToastFn).not.toHaveBeenCalled();
  });

  it('should handle error from orderAddCalldataFn', async () => {
    const error = new Error('Calldata failed');
    mockOrderAddCalldataFn.mockRejectedValue(error);
    mockFormatEthersTransactionErrorFn.mockReturnValue('Formatted: Calldata failed');

    await expect(
      executeWalletConnectOrder(mockDotrainText, mockDeployment, mockDependencies),
    ).rejects.toThrow(error);

    expect(mockReportErrorToSentryFn).toHaveBeenCalledWith(error);
    expect(mockErrorToastFn).toHaveBeenCalledWith('Formatted: Calldata failed');
    expect(mockSuccessToastFn).not.toHaveBeenCalled();
  });

  it('should handle error from ethersExecuteFn', async () => {
    const error = new Error('Ethers execute failed');
    mockOrderAddCalldataFn.mockResolvedValue(mockCalldata);
    mockEthersExecuteFn.mockRejectedValue(error);
    mockFormatEthersTransactionErrorFn.mockReturnValue('Formatted: Ethers failed');

    await expect(
      executeWalletConnectOrder(mockDotrainText, mockDeployment, mockDependencies),
    ).rejects.toThrow(error);

    expect(mockReportErrorToSentryFn).toHaveBeenCalledWith(error);
    expect(mockErrorToastFn).toHaveBeenCalledWith('Formatted: Ethers failed');
    expect(mockSuccessToastFn).not.toHaveBeenCalled(); // Success toast for sending should not be called
  });

  it('should handle error from tx.wait', async () => {
    const error = new Error('Wait failed');
    mockOrderAddCalldataFn.mockResolvedValue(mockCalldata);
    const failingTxResponse = { wait: vi.fn().mockRejectedValue(error) };
    mockEthersExecuteFn.mockResolvedValue(failingTxResponse);
    mockFormatEthersTransactionErrorFn.mockReturnValue('Formatted: Wait failed');

    await expect(
      executeWalletConnectOrder(mockDotrainText, mockDeployment, mockDependencies),
    ).rejects.toThrow(error);

    expect(mockSuccessToastFn).not.toHaveBeenCalled();
    expect(mockReportErrorToSentryFn).toHaveBeenCalledWith(error);
    expect(mockErrorToastFn).toHaveBeenCalledWith('Formatted: Wait failed');
  });
});
