import { useCallback, useEffect, useState } from 'react';
import {
  getProductionPreview,
  getProductionState,
  pauseCountdown,
  resumeCountdown,
  setCountdown,
  setAutoTransition,
  setCountdownSchedule,
  setCountdownRotation,
  createCountdown,
  updateCountdown,
  setMediaLive,
  setProductionMedia,
  startCountdown,
  stopCountdown,
} from '../lib/commands';
import type {
  CountdownDef, CountdownRotation, CountdownSchedule,
  ProductionPreview, ProductionSnapshot, TransitionTarget,
} from '../lib/types';

export function useProductionEngine(pollMs = 500) {
  const [snapshot, setSnapshot] = useState<ProductionSnapshot | null>(null);
  const [preview, setPreview] = useState<ProductionPreview | null>(null);
  const [error, setError] = useState('');

  const refresh = useCallback(async () => {
    try {
      const [state, prev] = await Promise.all([
        getProductionState(),
        getProductionPreview().catch(() => null),
      ]);
      setSnapshot(state);
      if (prev) setPreview(prev);
      setError('');
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, []);

  useEffect(() => {
    refresh();
    const timer = setInterval(refresh, pollMs);
    return () => clearInterval(timer);
  }, [pollMs, refresh]);

  const actions = {
    selectCountdown: async (id: string) => {
      const state = await setCountdown(id);
      setSnapshot(state);
      await refresh();
      return state;
    },
    startCountdown: async () => {
      const state = await startCountdown();
      setSnapshot(state);
      return state;
    },
    pauseCountdown: async () => {
      const state = await pauseCountdown();
      setSnapshot(state);
      return state;
    },
    resumeCountdown: async () => {
      const state = await resumeCountdown();
      setSnapshot(state);
      return state;
    },
    stopCountdown: async () => {
      const state = await stopCountdown();
      setSnapshot(state);
      return state;
    },
    selectMedia: async (id: string) => {
      const state = await setProductionMedia(id);
      setSnapshot(state);
      return state;
    },
    setMediaLive: async (live: boolean) => {
      const state = await setMediaLive(live);
      setSnapshot(state);
      return state;
    },
    setAutoTransition: async (enabled: boolean, target: TransitionTarget) => {
      const state = await setAutoTransition(enabled, target);
      setSnapshot(state);
      return state;
    },
    setCountdownSchedule: async (schedule: CountdownSchedule) => {
      const state = await setCountdownSchedule(schedule);
      setSnapshot(state);
      return state;
    },
    setCountdownRotation: async (rotation: CountdownRotation) => {
      const state = await setCountdownRotation(rotation);
      setSnapshot(state);
      return state;
    },
    createCountdown: async (def: CountdownDef) => {
      const state = await createCountdown(def);
      setSnapshot(state);
      return state;
    },
    updateCountdown: async (def: CountdownDef) => {
      const state = await updateCountdown(def);
      setSnapshot(state);
      return state;
    },
  };

  return { snapshot, preview, error, refresh, ...actions };
}
