import { render, screen } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import { goto } from '$app/navigation';
import DeploymentTile from '../lib/components/deployment/DeploymentTile.svelte';

// Mock the goto function
vi.mock('$app/navigation', () => ({
    goto: vi.fn()
}));

describe('DeploymentTile', () => {
    const mockProps = {
        strategyName: 'test-strategy',
        key: 'test-key',
        name: 'Test Deployment',
        description: 'This is a test deployment description'
    };

    it('renders the deployment name and description', () => {
        render(DeploymentTile, mockProps);

        expect(screen.getByText('Test Deployment')).toBeInTheDocument();
        expect(screen.getByText('This is a test deployment description')).toBeInTheDocument();
    });

    it('navigates to the correct URL when clicked', async () => {
        const { getByRole } = render(DeploymentTile, mockProps);

        const button = getByRole('button');
        await button.click();

        expect(goto).toHaveBeenCalledWith('/deploy/test-strategy/test-key');
    });
});