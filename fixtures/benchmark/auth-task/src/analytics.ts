export function trackPageView(route: string): string {
  return `page-view:${route}`;
}

export function trackError(message: string): string {
  return `error:${message}`;
}

export const analyticsDefaults = {
  sampleRate: 0.1,
  region: "local",
  retentionDays: 30,
};
