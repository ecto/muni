import posthog from "posthog-js";

export const POSTHOG_KEY = "phc_bJjV6mPT2NQSRZT1NciRlpUl5OdVgkH7AlJhdIY8ajT";
export const POSTHOG_HOST = "https://us.i.posthog.com";

export function initPostHog() {
  if (typeof window !== "undefined" && !posthog.__loaded) {
    posthog.init(POSTHOG_KEY, {
      api_host: POSTHOG_HOST,
      person_profiles: "identified_only",
      capture_pageview: false, // We'll capture manually for SPA
    });
  }
  return posthog;
}

export { posthog };
