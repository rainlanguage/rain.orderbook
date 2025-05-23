import type { DeploymentCfg } from '@rainlanguage/orderbook';
import { isEmpty } from 'lodash';
import type { SentrySeverityLevel } from './sentry';

/**
 * Executes the ledger order add process.
 *
 * @param dotrainText - The dotrain script text.
 * @param deployment - The selected deployment configuration.
 * @param orderAddFn - The function to call for adding the order (e.g., from order.ts service).
 * @param reportErrorToSentryFn - Function to report errors to Sentry.
 * @throws Error if deployment or orderbook details are missing.
 */
export async function executeLedgerOrder(
  dotrainText: string,
  deployment: DeploymentCfg,
  orderAddFn: (dotrain: string, deployment: DeploymentCfg) => Promise<void>,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  reportErrorToSentryFn: (error: any, level?: SentrySeverityLevel) => void,
): Promise<void> {
  if (isEmpty(deployment.order?.orderbook) || isEmpty(deployment.order.orderbook?.address))
    throw Error('No orderbook associated with scenario');

  try {
    await orderAddFn(dotrainText, deployment);
  } catch (e) {
    reportErrorToSentryFn(e);
    // Re-throw the error if the caller needs to handle it further or update UI state based on it
    // For now, just reporting, as the original component managed isSubmitting separately
    throw e;
  }
}
