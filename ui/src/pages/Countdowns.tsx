import { useEffect, useMemo, useState } from 'react';
import { AlarmClock, CalendarClock, CircleDashed, Layers, Monitor, Pause, Play, Radio, RotateCcw, Search, Sparkles, Square, Wand2 } from 'lucide-react';
import { useProductionEngine } from '../hooks/useProductionEngine';
import { exportCountdownPack, importCountdownPack, listProductionCountdowns } from '../lib/commands';
import type { TransitionTarget } from '../lib/types';
import type { CountdownDef } from '../lib/types';

type CountdownType = 'Service' | 'Worship' | 'Sermon' | 'Conference' | 'Youth' | 'Livestream';
type LoaderStyle = 'Circular' | 'Ring' | 'Progress Bar' | 'Minimal Line' | 'Segments' | 'Pulse' | 'Wave' | 'Particles';
type TypographyPreset = 'Modern Worship' | 'Conference' | 'Broadcast' | 'Minimal' | 'Youth' | 'Classic Church';

type CountdownTemplate = CountdownDef & {
  type: CountdownType;
  typography: TypographyPreset;
  media: string;
  output: string;
  gradient: string;
};

const THEME_GRADIENTS: Record<string, string> = {
  'worship-glow': 'from-blue-500 via-indigo-800 to-surface-950',
  'youth-energy': 'from-fuchsia-500 via-purple-800 to-surface-950',
  'conference-minimal': 'from-cyan-500 via-slate-800 to-surface-950',
  'prayer-soft': 'from-purple-500 via-fuchsia-900 to-surface-950',
  'blackout': 'from-black via-surface-950 to-black',
  'broadcast': 'from-red-500 via-surface-800 to-black',
  'classic-church': 'from-amber-300 via-orange-800 to-surface-950',
};

const TYPE_BY_ID: Record<string, CountdownType> = {
  'sunday-service': 'Service',
  'youth-night': 'Youth',
  'conference-session': 'Conference',
  'livestream': 'Livestream',
  'sermon': 'Sermon',
};

const COUNTDOWN_TYPES: CountdownType[] = ['Service', 'Worship', 'Sermon', 'Conference', 'Youth', 'Livestream'];
const LOADERS: LoaderStyle[] = ['Circular', 'Ring', 'Progress Bar', 'Minimal Line', 'Segments', 'Pulse', 'Wave', 'Particles'];
const TYPOGRAPHY: TypographyPreset[] = ['Modern Worship', 'Conference', 'Broadcast', 'Minimal', 'Youth', 'Classic Church'];

function toTemplate(def: CountdownDef): CountdownTemplate {
  return {
    ...def,
    type: TYPE_BY_ID[def.id] ?? 'Service',
    typography: def.theme_id === 'youth-energy' ? 'Youth'
      : def.theme_id === 'conference-minimal' ? 'Conference'
      : def.theme_id === 'broadcast' ? 'Broadcast'
      : def.theme_id === 'classic-church' ? 'Classic Church'
      : 'Modern Worship',
    media: def.media_id?.replace('-', ' ') ?? 'Theme background',
    output: 'Program',
    gradient: THEME_GRADIENTS[def.theme_id] ?? 'from-surface-700 to-surface-950',
  };
}

const THEME_FOR_TYPOGRAPHY: Record<TypographyPreset, string> = {
  'Modern Worship': 'worship-glow',
  'Conference': 'conference-minimal',
  'Broadcast': 'broadcast',
  'Minimal': 'blackout',
  'Youth': 'youth-energy',
  'Classic Church': 'classic-church',
};

function loaderToStyle(loader: LoaderStyle): CountdownDef['style'] {
  if (loader === 'Ring' || loader === 'Circular') return 'ring';
  if (['Pulse', 'Wave', 'Particles', 'Progress Bar', 'Minimal Line', 'Segments'].includes(loader)) return 'loader';
  return 'numeric';
}

function buildDef(template: CountdownTemplate, nextLoader: LoaderStyle, nextTypography: TypographyPreset): CountdownDef {
  return {
    id: template.id,
    name: template.name,
    duration: template.duration,
    style: loaderToStyle(nextLoader),
    theme_id: THEME_FOR_TYPOGRAPHY[nextTypography],
    headline: template.headline,
    subline: template.subline,
    loader: nextLoader,
    media_id: template.media_id ?? null,
  };
}

function formatTime(seconds: number) {
  const minutes = Math.floor(seconds / 60).toString().padStart(2, '0');
  const secs = (seconds % 60).toString().padStart(2, '0');
  return `${minutes}:${secs}`;
}

export default function Countdowns() {
  const production = useProductionEngine(400);
  const [templates, setTemplates] = useState<CountdownTemplate[]>([]);
  const [query, setQuery] = useState('');
  const [type, setType] = useState<CountdownType | 'All'>('All');
  const [selected, setSelected] = useState<CountdownTemplate | null>(null);
  const [loader, setLoader] = useState<LoaderStyle>('Ring');
  const [typography, setTypography] = useState<TypographyPreset>('Modern Worship');
  const scheduleStatus = production.snapshot?.schedule;
  const autoStart = scheduleStatus?.schedule.enabled ?? false;
  const leadSecs = scheduleStatus?.schedule.lead_secs ?? 600;
  const autoTransition = production.snapshot?.auto_transition ?? true;
  const transitionTarget = (production.snapshot?.transition_target ?? 'media') as TransitionTarget;
  const rotation = production.snapshot?.rotation;
  const rotationEnabled = rotation?.enabled ?? false;
  const rotationItems = rotation?.items ?? [];
  const [status, setStatus] = useState('');

  const refreshTemplates = async (pickId?: string) => {
    const defs = await listProductionCountdowns();
    const mapped = defs.map(toTemplate);
    setTemplates(mapped);
    const next = mapped.find(t => t.id === (pickId ?? selected?.id)) ?? mapped[0] ?? null;
    if (next) {
      setSelected(next);
      setLoader(next.loader as LoaderStyle);
      setTypography(next.typography);
    }
  };

  const applyCountdownOptions = async (nextLoader: LoaderStyle, nextTypography: TypographyPreset) => {
    if (!selected) return;
    await production.updateCountdown(buildDef(selected, nextLoader, nextTypography));
    await refreshTemplates(selected.id);
    setStatus(`Updated ${selected.name}`);
  };

  useEffect(() => {
    listProductionCountdowns()
      .then(defs => {
        const mapped = defs.map(toTemplate);
        setTemplates(mapped);
        if (!selected && mapped[0]) {
          setSelected(mapped[0]);
          setLoader(mapped[0].loader as LoaderStyle);
          setTypography(mapped[0].typography);
        }
      })
      .catch(() => setStatus('Failed to load countdown library'));
  }, []);

  const liveCountdown = production.snapshot?.countdown;
  const current = selected
    ? {
        ...selected,
        duration: liveCountdown?.def.id === selected.id
          ? liveCountdown.remaining_secs
          : selected.duration,
      }
    : null;

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return templates.filter(template => {
      const matchesType = type === 'All' || template.type === type;
      const matchesQuery = !q ||
        template.name.toLowerCase().includes(q) ||
        template.headline.toLowerCase().includes(q) ||
        template.type.toLowerCase().includes(q);
      return matchesType && matchesQuery;
    });
  }, [query, type, templates]);

  const selectTemplate = (template: CountdownTemplate) => {
    setSelected(template);
    setLoader(template.loader as LoaderStyle);
    setTypography(template.typography);
  };

  return (
    <div className="grid h-full grid-rows-[auto_1fr] bg-surface-900">
      <header className="border-b border-bdr bg-surface-950 px-5 py-4">
        <div className="flex items-center justify-between gap-4">
          <div>
            <p className="text-2xs font-black uppercase tracking-wider text-accent">Countdown Engine</p>
            <h1 className="mt-1 text-2xl font-black text-ink">Countdowns</h1>
            <p className="mt-1 max-w-3xl text-sm text-ink-faint">
              Build countdown experiences from timer, theme, media, typography, loaders, animation, scheduling, and output routing.
            </p>
          </div>
          <div className="flex gap-2">
            <button
              onClick={async () => {
                if (!selected) return;
                await applyCountdownOptions(loader, typography);
                setStatus('Theme saved to countdown');
              }}
              className="flex items-center gap-2 rounded-lg border border-bdr bg-surface-800 px-3 py-2 text-xs font-bold text-ink-faint hover:text-ink"
            >
              <Wand2 size={14} /> Save Theme
            </button>
            <button
              onClick={async () => {
                if (!selected) return;
                const id = `custom-${Date.now()}`;
                const def = {
                  ...buildDef(selected, loader, typography),
                  id,
                  name: `${selected.name} Copy`,
                };
                await production.createCountdown(def);
                await refreshTemplates(id);
                setStatus(`Created ${def.name}`);
              }}
              className="flex items-center gap-2 rounded-lg bg-accent px-3 py-2 text-xs font-black text-surface-950"
            >
              <AlarmClock size={14} /> New Countdown
            </button>
          </div>
        </div>
      </header>

      <div className="grid min-h-0 grid-cols-[280px_minmax(0,1fr)_380px]">
        <aside className="flex min-h-0 flex-col border-r border-bdr bg-surface-950">
          <div className="border-b border-bdr p-3">
            <div className="relative">
              <Search size={13} className="absolute left-3 top-1/2 -translate-y-1/2 text-accent" />
              <input
                value={query}
                onChange={event => setQuery(event.target.value)}
                placeholder="Search countdowns..."
                className="w-full rounded-lg border border-bdr bg-surface-800 py-2 pl-9 pr-3 text-xs font-semibold text-ink outline-none placeholder:text-ink-faint focus:border-accent"
              />
            </div>
          </div>

          <div className="border-b border-bdr p-3">
            <p className="mb-2 text-2xs font-black uppercase tracking-wider text-ink-faint">Types</p>
            <div className="grid grid-cols-2 gap-1">
              {(['All', ...COUNTDOWN_TYPES] as const).map(item => (
                <button
                  key={item}
                  onClick={() => setType(item)}
                  className={[
                    'rounded-lg px-2 py-2 text-xs font-bold transition-all',
                    type === item ? 'bg-accent text-surface-950' : 'bg-surface-800 text-ink-faint hover:text-ink',
                  ].join(' ')}
                >
                  {item}
                </button>
              ))}
            </div>
          </div>

          <div className="min-h-0 flex-1 overflow-y-auto p-3">
            <p className="mb-2 text-2xs font-black uppercase tracking-wider text-ink-faint">Countdown Library</p>
            <div className="space-y-2">
              {filtered.map(template => (
                <button
                  key={template.id}
                  onClick={() => selectTemplate(template)}
                  className={[
                    'w-full overflow-hidden rounded-xl border text-left transition-all hover:border-accent/50',
                    selected?.id === template.id ? 'border-accent bg-accent/10 shadow-glow' : 'border-bdr bg-surface-800',
                  ].join(' ')}
                >
                  <div className={`h-16 bg-gradient-to-br ${template.gradient}`} />
                  <div className="p-3">
                    <p className="text-sm font-black text-ink">{template.name}</p>
                    <p className="mt-1 text-xs text-ink-faint">{template.headline} - {formatTime(template.duration)}</p>
                  </div>
                </button>
              ))}
            </div>
          </div>
        </aside>

        <main className="min-h-0 overflow-y-auto p-5">
          {status && <p className="mb-3 text-xs font-bold text-accent">{status}</p>}
          {production.snapshot && (
            <p className="mb-3 text-xs text-ink-faint">
              Live layer: <span className="font-bold text-ink">{production.snapshot.active_layer}</span>
              {liveCountdown && (
                <> — {liveCountdown.status} ({formatTime(liveCountdown.remaining_secs)})</>
              )}
            </p>
          )}
          {selected && current && (
          <section className="mb-5 grid gap-4 lg:grid-cols-2">
            <div className="rounded-2xl border border-live/30 bg-live/5 p-4">
              <p className="mb-3 text-2xs font-black uppercase tracking-wider text-live">Program Output</p>
              {production.preview?.png_b64 ? (
                <img
                  src={`data:image/png;base64,${production.preview.png_b64}`}
                  alt="Live countdown output"
                  className="w-full rounded-xl border border-bdr"
                />
              ) : (
                <CountdownPreview label="Current" template={current} loader={current.loader as LoaderStyle} typography={current.typography} tone="live" />
              )}
            </div>
            <CountdownPreview label="Preview" template={selected} loader={loader} typography={typography} tone="accent" />
          </section>
          )}

          <section className="mb-5 grid gap-4 xl:grid-cols-3">
            <Panel title="Countdown Architecture" icon={<Layers size={14} />}>
              {['Countdown Theme', 'Countdown Timer', 'Media Background', 'Typography Engine', 'Animation Engine', 'Output Manager'].map((item, index) => (
                <div key={item} className="mb-2 flex items-center gap-2 rounded-lg bg-surface-900 px-3 py-2 text-xs text-ink-muted">
                  <span className="flex size-5 items-center justify-center rounded-full bg-accent/15 text-2xs font-black text-accent">{index + 1}</span>
                  {item}
                </div>
              ))}
            </Panel>

            <Panel title="Loader System" icon={<CircleDashed size={14} />}>
              <div className="grid grid-cols-2 gap-2">
                {LOADERS.map(item => (
                  <button
                    key={item}
                    onClick={async () => {
                      setLoader(item);
                      await applyCountdownOptions(item, typography);
                    }}
                    className={[
                      'rounded-lg border px-2 py-2 text-xs font-bold transition-all',
                      loader === item ? 'border-accent bg-accent/10 text-accent' : 'border-bdr bg-surface-900 text-ink-faint hover:text-ink',
                    ].join(' ')}
                  >
                    {item}
                  </button>
                ))}
              </div>
            </Panel>

            <Panel title="Typography Engine" icon={<Sparkles size={14} />}>
              <div className="space-y-2">
                {TYPOGRAPHY.map(item => (
                  <button
                    key={item}
                    onClick={async () => {
                      setTypography(item);
                      await applyCountdownOptions(loader, item);
                    }}
                    className={[
                      'flex w-full items-center justify-between rounded-lg border px-3 py-2 text-xs font-bold transition-all',
                      typography === item ? 'border-accent bg-accent/10 text-accent' : 'border-bdr bg-surface-900 text-ink-faint hover:text-ink',
                    ].join(' ')}
                  >
                    {item}
                    {typography === item && <span>Active</span>}
                  </button>
                ))}
              </div>
            </Panel>
          </section>

          <section className="grid gap-4 xl:grid-cols-2">
            <Panel title="Rotation Content" icon={<RotateCcw size={14} />}>
              <div className="mb-3 flex items-center justify-between rounded-lg border border-bdr bg-surface-900 px-3 py-2">
                <span className="text-xs font-bold text-ink-muted">Announcement + scripture rotation</span>
                <button
                  onClick={async () => {
                    await production.setCountdownRotation({
                      enabled: !rotationEnabled,
                      items: rotationItems,
                      interval_secs: rotation?.interval_secs ?? 8,
                    });
                    setStatus(!rotationEnabled ? 'Rotation enabled on live output' : 'Rotation disabled');
                  }}
                  className={['rounded px-2 py-1 text-2xs font-black', rotationEnabled ? 'bg-accent text-surface-950' : 'bg-surface-700 text-ink-faint'].join(' ')}
                >
                  {rotationEnabled ? 'On' : 'Off'}
                </button>
              </div>
              <div className="space-y-2">
                {rotationItems.map(item => (
                  <div key={item} className="rounded-lg bg-surface-900 px-3 py-2 text-xs text-ink-muted">{item}</div>
                ))}
                {rotationItems.length === 0 && (
                  <p className="text-xs text-ink-faint">No rotation items configured.</p>
                )}
              </div>
            </Panel>

            <Panel title="Scheduling + Auto Transition" icon={<CalendarClock size={14} />}>
              <div className="grid gap-3 sm:grid-cols-2">
                <ToggleCard
                  title="Scheduled Auto Start"
                  description={`Start ${Math.round(leadSecs / 60)} minutes before service time.`}
                  enabled={autoStart}
                  onClick={async () => {
                    if (!selected) return;
                    const sched = scheduleStatus?.schedule;
                    await production.setCountdownSchedule({
                      enabled: !autoStart,
                      countdown_id: selected.id,
                      service_at_unix: sched?.service_at_unix ?? Math.floor(Date.now() / 1000) + leadSecs,
                      lead_secs: leadSecs,
                      fired: false,
                    });
                    setStatus(!autoStart ? 'Schedule armed' : 'Schedule disabled');
                  }}
                />
                <ToggleCard
                  title="Auto Transition"
                  description={`At 00:00 transition to: ${transitionTarget}`}
                  enabled={autoTransition}
                  onClick={() => production.setAutoTransition(!autoTransition, transitionTarget)}
                />
              </div>
              <div className="mt-3 grid gap-2 sm:grid-cols-2">
                <label className="block text-2xs font-bold uppercase tracking-wider text-ink-faint">
                  Service time
                  <input
                    type="datetime-local"
                    className="mt-1 w-full rounded-lg border border-bdr bg-surface-900 px-3 py-2 text-xs text-ink"
                    value={scheduleStatus?.schedule.service_at_unix
                      ? new Date(scheduleStatus.schedule.service_at_unix * 1000).toISOString().slice(0, 16)
                      : ''}
                    onChange={async e => {
                      if (!selected || !e.target.value) return;
                      const unix = Math.floor(new Date(e.target.value).getTime() / 1000);
                      await production.setCountdownSchedule({
                        enabled: autoStart,
                        countdown_id: selected.id,
                        service_at_unix: unix,
                        lead_secs: leadSecs,
                        fired: scheduleStatus?.schedule.fired ?? false,
                      });
                    }}
                  />
                </label>
                <label className="block text-2xs font-bold uppercase tracking-wider text-ink-faint">
                  Lead time (minutes)
                  <input
                    type="number"
                    min={1}
                    max={120}
                    className="mt-1 w-full rounded-lg border border-bdr bg-surface-900 px-3 py-2 text-xs text-ink"
                    value={Math.round(leadSecs / 60)}
                    onChange={async e => {
                      if (!selected) return;
                      const mins = Math.max(1, Number(e.target.value) || 10);
                      await production.setCountdownSchedule({
                        enabled: autoStart,
                        countdown_id: selected.id,
                        service_at_unix: scheduleStatus?.schedule.service_at_unix ?? Math.floor(Date.now() / 1000),
                        lead_secs: mins * 60,
                        fired: scheduleStatus?.schedule.fired ?? false,
                      });
                    }}
                  />
                </label>
              </div>
              <div className="mt-3 grid grid-cols-3 gap-2">
                {(['media', 'idle', 'stop'] as const).map(target => (
                  <button
                    key={target}
                    onClick={() => production.setAutoTransition(autoTransition, target)}
                    className={[
                      'rounded-lg border px-2 py-2 text-xs font-bold capitalize',
                      transitionTarget === target ? 'border-accent bg-accent/10 text-accent' : 'border-bdr text-ink-faint',
                    ].join(' ')}
                  >
                    {target}
                  </button>
                ))}
              </div>
              <div className="mt-3 rounded-xl border border-bdr bg-surface-900 p-3">
                <p className="text-2xs font-black uppercase tracking-wider text-ink-faint">Schedule status</p>
                {scheduleStatus?.schedule.fired ? (
                  <p className="mt-2 text-sm font-black text-accent">Auto-start fired for this service</p>
                ) : scheduleStatus?.ready ? (
                  <p className="mt-2 text-sm font-black text-live">Ready — countdown will start on next tick</p>
                ) : (
                  <p className="mt-2 text-sm font-black text-ink">
                    {scheduleStatus && scheduleStatus.seconds_until_start > 0
                      ? `Starts in ${formatTime(scheduleStatus.seconds_until_start)}`
                      : 'Set a service time to arm the schedule'}
                  </p>
                )}
                <p className="mt-1 text-xs text-ink-faint">
                  Uses {selected?.name ?? 'selected countdown'} with {Math.round(leadSecs / 60)} min lead.
                </p>
              </div>
            </Panel>
          </section>
        </main>

        <aside className="flex min-h-0 flex-col border-l border-bdr bg-surface-950">
          {selected && (
          <section className="border-b border-bdr p-4">
            <p className="text-2xs font-black uppercase tracking-wider text-ink-faint">Selected Countdown</p>
            <div className={`mt-3 rounded-2xl bg-gradient-to-br ${selected.gradient} p-5 text-center`}>
              <p className="text-sm font-black uppercase tracking-[0.24em] text-white/80">{selected.headline}</p>
              <p className="mt-3 text-5xl font-black text-white drop-shadow">
                {formatTime(liveCountdown?.def.id === selected.id ? liveCountdown.remaining_secs : selected.duration)}
              </p>
              <p className="mt-3 text-sm font-semibold text-white/80">{selected.subline}</p>
            </div>
            <h2 className="mt-4 text-xl font-black text-ink">{selected.name}</h2>
            <p className="mt-1 text-xs text-ink-faint">{selected.type} countdown using {selected.media}.</p>
            <div className="mt-3 grid grid-cols-2 gap-2">
              <button
                onClick={async () => {
                  await production.selectCountdown(selected.id);
                  setStatus(`Loaded ${selected.name}`);
                }}
                className="flex items-center justify-center gap-2 rounded-lg border border-bdr bg-surface-800 px-3 py-2 text-xs font-bold text-ink-faint hover:text-ink"
              >
                Load
              </button>
              <button
                onClick={async () => {
                  await production.selectCountdown(selected.id);
                  await production.startCountdown();
                  setStatus(`Live: ${selected.name}`);
                }}
                className="rounded-lg bg-accent px-3 py-2 text-xs font-black text-surface-950"
              >
                Start
              </button>
            </div>
            <div className="mt-2 grid grid-cols-3 gap-2">
              <button
                onClick={() => production.pauseCountdown().then(() => setStatus('Paused'))}
                className="flex items-center justify-center gap-1 rounded-lg border border-bdr bg-surface-800 px-2 py-2 text-2xs font-bold text-ink-faint hover:text-ink"
              >
                <Pause size={12} /> Pause
              </button>
              <button
                onClick={() => production.resumeCountdown().then(() => setStatus('Resumed'))}
                className="flex items-center justify-center gap-1 rounded-lg border border-bdr bg-surface-800 px-2 py-2 text-2xs font-bold text-ink-faint hover:text-ink"
              >
                <Play size={12} /> Resume
              </button>
              <button
                onClick={() => production.stopCountdown().then(() => setStatus('Stopped'))}
                className="flex items-center justify-center gap-1 rounded-lg border border-bdr bg-surface-800 px-2 py-2 text-2xs font-bold text-ink-faint hover:text-ink"
              >
                <Square size={12} /> Stop
              </button>
            </div>
          </section>
          )}

          <section className="min-h-0 flex-1 overflow-y-auto p-4">
            <p className="mb-3 flex items-center gap-2 text-2xs font-black uppercase tracking-wider text-ink-faint">
              <Monitor size={13} /> Multi-Output Layouts
            </p>
            <div className="space-y-3">
              {[
                ['Program Output', 'Full countdown theme with motion media and loader.'],
                ['Stage Display', 'Countdown + live clock with high contrast.'],
                ['Confidence Monitor', 'Countdown, next service cue, and operator notes.'],
                ['Livestream', 'Broadcast-safe countdown with branding and lower third space.'],
              ].map(([label, description]) => (
                <div key={label} className="rounded-xl border border-bdr bg-surface-800 p-3">
                  <p className="text-sm font-black text-ink">{label}</p>
                  <p className="mt-1 text-xs leading-relaxed text-ink-faint">{description}</p>
                </div>
              ))}
            </div>
          </section>

          <section className="border-t border-bdr p-4">
            <p className="mb-2 flex items-center gap-2 text-2xs font-black uppercase tracking-wider text-ink-faint">
              <Radio size={13} /> Package Format
            </p>
            <p className="mb-2 text-xs text-ink-faint">
              Packs use <span className="font-mono text-accent">.bpcountdown</span> JSON with theme and configuration.
            </p>
            <div className="grid grid-cols-2 gap-2">
              <button
                onClick={async () => {
                  if (!selected) return;
                  const json = await exportCountdownPack(selected.id);
                  await navigator.clipboard.writeText(json);
                  setStatus('Pack copied to clipboard');
                }}
                className="rounded-lg border border-bdr bg-surface-800 px-2 py-2 text-2xs font-bold text-ink-faint hover:text-ink"
              >
                Export
              </button>
              <button
                onClick={async () => {
                  const json = prompt('Paste .bpcountdown JSON');
                  if (!json) return;
                  await importCountdownPack(json);
                  setStatus('Pack imported');
                  listProductionCountdowns().then(defs => setTemplates(defs.map(toTemplate)));
                }}
                className="rounded-lg border border-bdr bg-surface-800 px-2 py-2 text-2xs font-bold text-ink-faint hover:text-ink"
              >
                Import
              </button>
            </div>
          </section>
        </aside>
      </div>
    </div>
  );
}

function CountdownPreview({
  label,
  template,
  loader,
  typography,
  tone,
}: {
  label: string;
  template: CountdownTemplate;
  loader: LoaderStyle;
  typography: TypographyPreset;
  tone: 'live' | 'accent';
}) {
  const toneClass = tone === 'live' ? 'border-live/30 bg-live/5 text-live' : 'border-accent/30 bg-accent/5 text-accent';

  return (
    <div className={`rounded-2xl border p-4 ${toneClass}`}>
      <div className="mb-3 flex items-center justify-between">
        <p className="text-2xs font-black uppercase tracking-wider">{label}</p>
        <span className="rounded-full bg-surface-900 px-2 py-0.5 text-2xs font-bold text-ink-faint">{loader}</span>
      </div>
      <div className={`relative flex min-h-[260px] flex-col items-center justify-center overflow-hidden rounded-2xl bg-gradient-to-br ${template.gradient} p-6 text-center`}>
        <div className="absolute inset-0 bg-black/25" />
        <LoaderVisual loader={loader} />
        <div className="relative z-10">
          <p className="text-sm font-black uppercase tracking-[0.26em] text-white/80">{template.headline}</p>
          <p className={['mt-4 text-7xl text-white drop-shadow', typography === 'Minimal' ? 'font-thin' : 'font-black'].join(' ')}>
            {formatTime(template.duration)}
          </p>
          <p className="mt-4 text-sm font-semibold text-white/80">{template.subline}</p>
        </div>
      </div>
      <p className="mt-3 text-sm font-black text-ink">{template.name}</p>
      <p className="mt-1 text-xs text-ink-faint">{typography} - {template.media}</p>
    </div>
  );
}

function LoaderVisual({ loader }: { loader: LoaderStyle }) {
  if (loader === 'Progress Bar') {
    return <div className="absolute bottom-0 left-0 z-10 h-1.5 w-3/4 rounded-r-full bg-white/80" />;
  }
  if (loader === 'Minimal Line') {
    return <div className="absolute bottom-12 z-10 h-px w-2/3 bg-white/60" />;
  }
  if (loader === 'Segments') {
    return (
      <div className="absolute bottom-10 z-10 flex gap-1">
        {Array.from({ length: 10 }, (_, index) => <span key={index} className={['h-2 w-5 rounded-sm', index < 6 ? 'bg-white/80' : 'bg-white/20'].join(' ')} />)}
      </div>
    );
  }
  if (loader === 'Wave') {
    return (
      <div className="absolute bottom-10 z-10 flex items-end gap-1">
        {Array.from({ length: 18 }, (_, index) => <span key={index} className="w-1 rounded-full bg-white/60" style={{ height: `${10 + (index % 5) * 5}px` }} />)}
      </div>
    );
  }
  if (loader === 'Pulse') {
    return <div className="absolute z-0 size-56 rounded-full border border-white/20 bg-white/5" />;
  }
  return <CircleDashed size={190} className="absolute z-0 text-white/15" />;
}

function Panel({ title, icon, children }: { title: string; icon: React.ReactNode; children: React.ReactNode }) {
  return (
    <div className="rounded-2xl border border-bdr bg-surface-800 p-4">
      <p className="mb-3 flex items-center gap-2 text-2xs font-black uppercase tracking-wider text-ink-faint">
        {icon} {title}
      </p>
      {children}
    </div>
  );
}

function ToggleCard({ title, description, enabled, onClick }: { title: string; description: string; enabled: boolean; onClick: () => void }) {
  return (
    <button onClick={onClick} className="rounded-xl border border-bdr bg-surface-900 p-3 text-left">
      <div className="flex items-center justify-between gap-3">
        <p className="text-sm font-black text-ink">{title}</p>
        <span className={['rounded px-2 py-1 text-2xs font-black', enabled ? 'bg-accent text-surface-950' : 'bg-surface-700 text-ink-faint'].join(' ')}>
          {enabled ? 'On' : 'Off'}
        </span>
      </div>
      <p className="mt-2 text-xs leading-relaxed text-ink-faint">{description}</p>
    </button>
  );
}
