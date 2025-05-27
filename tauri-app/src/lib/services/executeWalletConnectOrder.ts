import type { DeploymentCfg } from '@rainlanguage/orderbook';
import { isEmpty } from 'lodash';
import type { SentrySeverityLevel } from './sentry';

interface EthersTransactionResponse {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  wait: (confirmations: number) => Promise<any>;
}

/**
 * Executes the WalletConnect order add process.
 *
 * @param dotrainText - The dotrain script text.
 * @param deployment - The selected deployment configuration.
 * @param orderAddCalldataFn - Function to get order calldata.
 * @param ethersExecuteFn - Function to execute the transaction via ethers.
 * @param reportErrorToSentryFn - Function to report errors to Sentry.
 * @param formatEthersTransactionErrorFn - Function to format ethers transaction errors.
 * @param successToastFn - Function to show a success toast.
 * @param errorToastFn - Function to show an error toast.
 * @throws Error if deployment or orderbook details are missing.
 */
export async function executeWalletConnectOrder(
  dotrainText: string,
  deployment: DeploymentCfg,
  orderAddCalldataFn: (dotrain: string, deployment: DeploymentCfg) => Promise<Uint8Array>,
  ethersExecuteFn: (calldata: Uint8Array, toAddress: string) => Promise<EthersTransactionResponse>,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  reportErrorToSentryFn: (error: any, level?: SentrySeverityLevel) => void,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  formatEthersTransactionErrorFn: (error: any) => string,
  successToastFn: (message: string) => void,
  errorToastFn: (message: string) => void,
): Promise<void> {
  if (isEmpty(deployment.order.orderbook))
    throw Error('No orderbook associated with scenario');
  try {
    const calldata = await orderAddCalldataFn(dotrainText, deployment);
    const tx = await ethersExecuteFn(calldata, deployment.order.orderbook.address);
    successToastFn('Transaction sent successfully!');
    await tx.wait(1);
  } catch (e) {
    reportErrorToSentryFn(e);
    errorToastFn(formatEthersTransactionErrorFn(e));
    throw e;
  }
}
