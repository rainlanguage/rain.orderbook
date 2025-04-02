import type { SgOrder } from '@rainlanguage/orderbook/js_api';
import type { Hex } from 'viem';

/**
 * Pure function that takes order data and returns the configuration needed
 * for order removal without performing any side effects
 * 
 * @param order The order to be potentially removed
 * @param config Additional configuration parameters
 * @returns Configuration object with all necessary data for the removal operation
 */
export const prepareOrderRemoval = (
  order: SgOrder,
  config: {
    chainId: number;
    orderbookAddress: Hex;
    subgraphUrl: string;
  }
) => {
  // Return a data structure describing what should happen
  // without actually performing any side effects
  return {
    modal: {
      open: true,
      args: {
        order,
        chainId: config.chainId,
        orderbookAddress: config.orderbookAddress,
        subgraphUrl: config.subgraphUrl
      }
    },
    queryInvalidation: {
      queryKey: [order.orderHash],
      refetchType: 'all' as const,
      exact: false
    },
    notification: 'Order removed successfully'
  };
};