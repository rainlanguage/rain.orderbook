import { handleDeployModal } from './modal';

import type { DeploymentArgs } from '@rainlanguage/ui-components';
import { handleDisclaimerModal } from './modal';

export function handleDeploy(deploymentArgs: DeploymentArgs) {
	handleDisclaimerModal({
		open: true,
		onAccept: () => {
			handleDeployModal({
				args: deploymentArgs,
				open: true
			});
		}
	});
}
