import type { DeploymentCfg } from '@rainlanguage/orderbook';
import { isEmpty } from 'lodash';
import type { SentrySeverityLevel } from './sentry';
import { ethers } from 'ethers';

interface EthersTransactionResponse {
  wait: (confirmations: number) => Promise<ethers.providers.TransactionReceipt>;
}

export interface ExecuteOrderDependencies {
  orderAddCalldataFn: (dotrain: string, deployment: DeploymentCfg) => Promise<Uint8Array>;
  ethersExecuteFn: (calldata: Uint8Array, toAddress: string) => Promise<EthersTransactionResponse>;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  reportErrorToSentryFn: (error: any, level?: SentrySeverityLevel) => void;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  formatEthersTransactionErrorFn: (error: any) => string;
  successToastFn: (message: string) => void;
  errorToastFn: (message: string) => void;
}

/**
 * Executes the WalletConnect order add process.
 *
 * @param dotrainText - The dotrain script text.
 * @param deployment - The selected deployment configuration.
 * @param dependencies - Object containing all dependencies for execution and feedback.
 * @throws Error if deployment or orderbook details are missing.
 */
export async function executeWalletConnectOrder(
  dotrainText: string,
  deployment: DeploymentCfg,
  dependencies: ExecuteOrderDependencies,
): Promise<void> {
  const {
    orderAddCalldataFn,
    ethersExecuteFn,
    reportErrorToSentryFn,
    formatEthersTransactionErrorFn,
    successToastFn,
    errorToastFn,
  } = dependencies;
  if (isEmpty(deployment.order.orderbook)) throw Error('No orderbook associated with scenario');
  try {
    const calldata = await orderAddCalldataFn(dotrainText, deployment);
    const tx = await ethersExecuteFn(calldata, deployment.order.orderbook.address);
    await tx.wait(1);
    successToastFn('Transaction sent successfully!');
  } catch (e) {
    reportErrorToSentryFn(e);
    errorToastFn(formatEthersTransactionErrorFn(e));
    throw e;
  }
}
