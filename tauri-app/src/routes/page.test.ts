import { render, screen } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import Page from './+page.svelte';
import { logoDark, logoLight } from '@rainlanguage/ui-components';

const { mockColorThemeStore } = await vi.hoisted(() => import('../lib/__mocks__/stores'));

vi.mock('$lib/stores/darkMode', () => ({
  colorTheme: mockColorThemeStore,
}));

describe('+page.svelte', () => {
  beforeEach(() => {
    mockColorThemeStore.mockSetSubscribeValue('light');
  });

  it('renders the light logo when theme is light', () => {
    render(Page);
    const logo = screen.getByTestId('logo') as HTMLImageElement;
    expect(logo).toBeInTheDocument();
    expect(logo.src).toContain(logoLight);
  });

  it('renders the dark logo when theme is dark', () => {
    mockColorThemeStore.mockSetSubscribeValue('dark');
    render(Page);
    const logo = screen.getByTestId('logo') as HTMLImageElement;
    expect(logo).toBeInTheDocument();
    expect(logo.src).toContain(logoDark);
  });

  it('renders the main description text', () => {
    render(Page);
    expect(screen.getByTestId('description')).toBeInTheDocument();
    expect(screen.getByTestId('description').textContent).toContain(
      'Raindex allows anyone to write, deploy and manage token trading strategies',
    );
  });

  it('renders the "Join the community" button/link correctly', () => {
    render(Page);
    const communityLink = screen.getByTestId('community-link');
    expect(communityLink).toBeInTheDocument();
    expect(communityLink).toHaveAttribute('href', 'https://t.me/+W0aQ36ptN_E2MjZk');
    expect(communityLink).toHaveAttribute('target', '_blank');
    const svgIcon = communityLink.querySelector('svg');
    expect(svgIcon).toBeInTheDocument();
  });

  it('renders the "Get started" button/link correctly', () => {
    render(Page);
    const getStartedLink = screen.getByTestId('get-started-link');
    expect(getStartedLink).toBeInTheDocument();
    expect(getStartedLink).toHaveAttribute(
      'href',
      'https://docs.rainlang.xyz/raindex/getting-started',
    );
    expect(getStartedLink).toHaveAttribute('target', '_blank');
  });

  it('renders the Loom video iframe correctly', () => {
    render(Page);
    const iframe = screen.getByTestId('demo-iframe') as HTMLIFrameElement;
    expect(iframe).toBeInTheDocument();
    expect(iframe.src).toContain('https://www.loom.com/embed/fca750f31f0a43258891cea0ddacb588');
    expect(iframe).toHaveAttribute('allowfullscreen');
    expect(iframe).toHaveAttribute('title', 'Raindex Demo');
  });
});
