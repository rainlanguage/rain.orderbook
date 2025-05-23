import { describe, it, expect, vi, beforeEach } from 'vitest';
import { executeLedgerOrder } from '../services/executeLedgerOrder';
import type { DeploymentCfg } from '@rainlanguage/orderbook';


// Mocks
const mockOrderAddFn = vi.fn();
const mockReportErrorToSentryFn = vi.fn();

const mockDotrainText = 'some dotrain text';
const mockDeployment: DeploymentCfg = {
  order: {
    orderbook: { address: '0x123' },
  },
  // Add other necessary DeploymentCfg fields if your function uses them
} as DeploymentCfg;

describe('executeLedgerOrder', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should throw error if deployment is undefined', async () => {
    await expect(
      executeLedgerOrder(mockDotrainText, undefined, mockOrderAddFn, mockReportErrorToSentryFn),
    ).rejects.toThrow('Select a deployment to add order');
    expect(mockOrderAddFn).not.toHaveBeenCalled();
    expect(mockReportErrorToSentryFn).not.toHaveBeenCalled();
  });

  it('should throw error if orderbook address is missing', async () => {
    const deploymentWithoutOrderbook: DeploymentCfg = {
      order: { orderbook: {} }, // Missing address
    } as DeploymentCfg;
    await expect(
      executeLedgerOrder(
        mockDotrainText,
        deploymentWithoutOrderbook,
        mockOrderAddFn,
        mockReportErrorToSentryFn,
      ),
    ).rejects.toThrow('No orderbook associated with scenario');
  });

  it('should call orderAddFn with correct parameters on success', async () => {
    await executeLedgerOrder(
      mockDotrainText,
      mockDeployment,
      mockOrderAddFn,
      mockReportErrorToSentryFn,
    );
    expect(mockOrderAddFn).toHaveBeenCalledWith(mockDotrainText, mockDeployment);
    expect(mockReportErrorToSentryFn).not.toHaveBeenCalled();
  });

  it('should call reportErrorToSentryFn and re-throw if orderAddFn throws', async () => {
    const error = new Error('Order add failed');
    mockOrderAddFn.mockRejectedValueOnce(error);

    await expect(
      executeLedgerOrder(
        mockDotrainText,
        mockDeployment,
        mockOrderAddFn,
        mockReportErrorToSentryFn,
      ),
    ).rejects.toThrow(error);

    expect(mockOrderAddFn).toHaveBeenCalledWith(mockDotrainText, mockDeployment);
    expect(mockReportErrorToSentryFn).toHaveBeenCalledWith(error);
  });
});
