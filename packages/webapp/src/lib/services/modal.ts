

import DeployModal from '$lib/components/DeployModal.svelte';

export const handleDeployModal = () => {
  new DeployModal({ target: document.body, props: { open: true } });
};



