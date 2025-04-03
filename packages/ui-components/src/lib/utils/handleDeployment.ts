import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import type { HandleAddOrderResult } from '../components/deployment/getDeploymentTransactionArgs';
import { getDeploymentTransactionArgs } from '../components/deployment/getDeploymentTransactionArgs';

export interface DeploymentHandlers {
	handleDisclaimerModal?: (props: { open: boolean; onAccept: () => void }) => void;
	handleDeployModal: (props: { open: boolean; args: any }) => void;
}

export async function handleDeployment(
	gui: DotrainOrderGui,
	account: string | null,
	handlers: DeploymentHandlers,
	subgraphUrl?: string
): Promise<void> {
	try {
		const result: HandleAddOrderResult = await getDeploymentTransactionArgs(gui, account);
		if (handlers.handleDisclaimerModal) {
			handlers.handleDisclaimerModal({
				open: true,
				onAccept: () => {
					handlers.handleDeployModal({
						open: true,
						args: {
							...result,
							subgraphUrl
						}
					});
				}
			});
		} else {
			handlers.handleDeployModal({
				open: true,
				args: {
					...result,
					subgraphUrl
				}
			});
		}
	} catch (e) {
		throw e;
	}
}
