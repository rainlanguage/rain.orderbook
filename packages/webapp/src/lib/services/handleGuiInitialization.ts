import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

export async function handleGuiInitialization(
    dotrain: string,
    deploymentKey: string,
    stateFromUrl: string | null
): Promise<{ gui: DotrainOrderGui | null; error: string | null }> {
    try {
        let gui: DotrainOrderGui | null = null;
        
        if (stateFromUrl) {
            try {
                gui = await DotrainOrderGui.deserializeState(
                    dotrain,
                    stateFromUrl
                );
            } catch (deserializeErr) {
                gui = await DotrainOrderGui.chooseDeployment(dotrain, deploymentKey);
            }
        } else {
            gui = await DotrainOrderGui.chooseDeployment(dotrain, deploymentKey);
        }
        return { gui, error: null };
    } catch (err) {
        return { gui: null, error: 'Could not get deployment form.' };
    }
} 