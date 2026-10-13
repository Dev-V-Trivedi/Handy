/** Tiny shared store: the app shell records a detected call, the Meetings tab shows it. */
let detected: string | null = null;
const subscribers = new Set<() => void>();

export const getDetectedCall = (): string | null => detected;

export const setDetectedCall = (app: string | null): void => {
  detected = app;
  subscribers.forEach((fn) => fn());
};

export const subscribeDetectedCall = (fn: () => void): (() => void) => {
  subscribers.add(fn);
  return () => {
    subscribers.delete(fn);
  };
};
