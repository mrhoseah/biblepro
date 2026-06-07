import { useEffect, useMemo, useState } from 'react';
import { Activity, Radio, Eye, Monitor, Plus, RefreshCw, Route, Settings2, Shield, Smartphone, X } from 'lucide-react';
import { addDisplayOutput, addNdiOutput, getOutputs, listMonitors, removeOutput, setOutputLayout, setOutputRole, setOutputSource, setScriptureMode, toggleOutput } from '../lib/commands';
import type { MonitorInfo, OutputInfo, OutputRole, OutputSource, RoleLayout, ScriptureMode } from '../lib/types';

const ROLE_LAYOUTS: { id: RoleLayout; label: string }[] = [
  { id: 'auto', label: 'Auto (role default)' },
  { id: 'full', label: 'Full frame' },
  { id: 'stage_timer', label: 'Stage timer' },
  { id: 'confidence_text', label: 'Confidence text' },
  { id: 'lobby_countdown', label: 'Lobby countdown' },
  { id: 'livestream_safe', label: 'Livestream safe zone' },
];

const OUTPUT_ROLES = [
  { id: 'program', label: 'Program Output', icon: Monitor, route: 'Scripture, songs, presentations', stack: ['Emergency', 'Scripture', 'Presentation', 'Media Theme'] },
  { id: 'preview', label: 'Preview Output', icon: Eye, route: 'Current, next, queue, live preview', stack: ['Next', 'Current', 'Queue', 'Media Theme'] },
  { id: 'confidence', label: 'Confidence Monitor', icon: Shield, route: 'Current verse, next verse, clock', stack: ['Notes', 'Timer', 'Current Verse', 'Solid Theme'] },
  { id: 'stage', label: 'Stage Display', icon: Activity, route: 'Current lyrics, next lyrics, scripture', stack: ['Current Lyrics', 'Next Lyrics', 'Readable Theme'] },
  { id: 'lobby', label: 'Lobby Display', icon: Smartphone, route: 'Announcements and countdowns', stack: ['Announcements', 'Countdown', 'Branding'] },
  { id: 'livestream', label: 'Livestream Output', icon: Radio, route: 'Lower thirds and stream graphics', stack: ['Lower Thirds', 'Branding', 'Transparent BG'] },
] as const;

function kindLabel(output: OutputInfo) {
  if (output.kind.type === 'display') return output.kind.monitor_name;
  return output.kind.source_name;
}

export default function Outputs() {
  const [monitors, setMonitors] = useState<MonitorInfo[]>([]);
  const [outputs, setOutputs] = useState<OutputInfo[]>([]);
  const [status, setStatus] = useState('');
  const [selectedRole, setSelectedRole] = useState<(typeof OUTPUT_ROLES)[number]['id']>('program');
  const [showNdiForm, setShowNdiForm] = useState(false);
  const [ndiLabel, setNdiLabel] = useState('Program NDI');
  const [ndiSource, setNdiSource] = useState('BiblePro Program');
  const [scriptureMode, setScriptureModeState] = useState<ScriptureMode>('replace');
  const [showConfigure, setShowConfigure] = useState(false);

  const defaultLayoutForRole = (role: string): RoleLayout => {
    switch (role) {
      case 'stage': return 'stage_timer';
      case 'confidence': return 'confidence_text';
      case 'lobby': return 'lobby_countdown';
      case 'livestream': return 'livestream_safe';
      default: return 'full';
    }
  };

  const refresh = async () => {
    const [mons, outs] = await Promise.all([listMonitors(), getOutputs()]);
    setMonitors(mons);
    setOutputs(outs);
  };

  useEffect(() => { refresh(); }, []);

  const assignedMonitors = useMemo(() => {
    const map = new Map<number, string>();
    outputs.forEach(output => {
      if (output.kind.type === 'display') {
        map.set(output.kind.monitor_index, output.id);
      }
    });
    return map;
  }, [outputs]);

  const enabledCount = outputs.filter(output => output.enabled).length;
  const selectedRoleInfo = OUTPUT_ROLES.find(role => role.id === selectedRole) ?? OUTPUT_ROLES[0];

  const openDisplay = async (monitor: MonitorInfo) => {
    const roleLabel = selectedRoleInfo.label;
    const label = `${roleLabel} - ${monitor.name}`;
    try {
      await addDisplayOutput(label, monitor.index, monitor.name, monitor.x, monitor.y, monitor.width, monitor.height, selectedRole);
      setStatus(`${roleLabel} opened on ${monitor.name}`);
      await refresh();
    } catch (error: any) {
      setStatus(error?.toString() ?? 'Failed to open display');
    }
  };

  const startNdi = async () => {
    try {
      await addNdiOutput(ndiLabel, ndiSource, selectedRole);
      setShowNdiForm(false);
      setStatus(`${ndiLabel} published as ${ndiSource}`);
      await refresh();
    } catch (error: any) {
      setStatus(error?.toString() ?? 'Failed to start NDI output');
    }
  };

  return (
    <div className="h-full overflow-y-auto bg-surface-900 p-6">
      <div className="mx-auto max-w-6xl space-y-6">
        <header className="flex items-start justify-between gap-4">
          <div>
            <p className="text-2xs font-black uppercase tracking-[0.22em] text-accent">Enterprise Output Manager</p>
            <h1 className="mt-1 text-2xl font-black text-ink">Outputs</h1>
            <p className="mt-1 max-w-2xl text-sm text-ink-faint">
              Treat every projector, monitor, NDI feed, and future web/mobile display as an independent output with its own audience, media theme, and layout.
            </p>
          </div>
          <button
            onClick={refresh}
            className="flex items-center gap-2 rounded-lg border border-bdr bg-surface-700 px-3 py-2 text-xs font-bold text-ink-muted transition-colors hover:bg-surface-600 hover:text-ink"
          >
            <RefreshCw size={13} /> Refresh
          </button>
        </header>

        {status && (
          <div className="rounded-lg border border-accent/30 bg-accent/10 px-4 py-2.5 text-sm text-accent">
            {status}
          </div>
        )}

        <section className="grid gap-4 md:grid-cols-3">
          <div className="rounded-xl border border-bdr bg-surface-800 p-4">
            <p className="text-2xs font-bold uppercase tracking-wider text-ink-faint">Live outputs</p>
            <p className="mt-2 text-3xl font-black text-accent">{enabledCount}</p>
            <p className="text-xs text-ink-faint">{outputs.length} configured</p>
          </div>
          <div className="rounded-xl border border-bdr bg-surface-800 p-4">
            <p className="text-2xs font-bold uppercase tracking-wider text-ink-faint">Physical monitors</p>
            <p className="mt-2 text-3xl font-black text-ink">{monitors.length}</p>
            <p className="text-xs text-ink-faint">Detected by BiblePro</p>
          </div>
          <div className="rounded-xl border border-bdr bg-surface-800 p-4">
            <p className="text-2xs font-bold uppercase tracking-wider text-ink-faint">Routing model</p>
            <p className="mt-2 text-sm font-bold text-ink">Media + scripture priority</p>
            <p className="mt-1 text-xs text-ink-faint">Themes provide media and layout; scripture can replace or overlay the current presentation.</p>
          </div>
        </section>

        <section className="grid gap-6 lg:grid-cols-[360px_1fr]">
          <div className="inset-panel overflow-hidden">
            <div className="border-b border-bdr px-4 py-3">
              <p className="text-xs font-bold uppercase tracking-wider text-ink">Output roles</p>
              <p className="mt-0.5 text-2xs text-ink-faint">Choose the audience you are configuring.</p>
            </div>
            <div className="divide-y divide-bdr/50">
              {OUTPUT_ROLES.map(role => {
                const Icon = role.icon;
                const active = selectedRole === role.id;
                return (
                  <button
                    key={role.id}
                    onClick={() => {
                      setSelectedRole(role.id);
                      setNdiLabel(`${role.label} NDI`);
                      setNdiSource(`BiblePro ${role.label}`);
                    }}
                    className={[
                      'flex w-full items-start gap-3 px-4 py-3 text-left transition-all',
                      active ? 'bg-accent/10 text-ink' : 'text-ink-faint hover:bg-surface-700 hover:text-ink',
                    ].join(' ')}
                  >
                    <div className={['mt-0.5 flex size-9 items-center justify-center rounded-lg border', active ? 'border-accent bg-accent text-surface-950' : 'border-bdr bg-surface-800'].join(' ')}>
                      <Icon size={16} />
                    </div>
                    <div>
                      <div className="text-sm font-bold">{role.label}</div>
                      <div className="mt-0.5 text-2xs opacity-75">{role.route}</div>
                    </div>
                  </button>
                );
              })}
            </div>
          </div>

          <div className="space-y-6">
            <section className="inset-panel overflow-hidden">
              <div className="flex items-center justify-between border-b border-bdr px-4 py-3">
                <div>
                  <p className="text-xs font-bold uppercase tracking-wider text-ink">{selectedRoleInfo.label}</p>
                  <p className="mt-0.5 text-2xs text-ink-faint">{selectedRoleInfo.route}</p>
                </div>
                <button
                  onClick={() => setShowConfigure(v => !v)}
                  className={['flex items-center gap-1.5 rounded-md border px-3 py-1.5 text-2xs font-bold', showConfigure ? 'border-accent bg-accent/10 text-accent' : 'border-bdr bg-surface-700 text-ink-faint'].join(' ')}
                >
                  <Settings2 size={11} /> Configure
                </button>
              </div>
              {showConfigure && (
                <div className="border-b border-bdr bg-surface-900 px-4 py-3">
                  <p className="text-2xs font-bold uppercase tracking-wider text-ink-faint">Role defaults</p>
                  <p className="mt-1 text-xs text-ink-muted">Apply the recommended layout to all {selectedRoleInfo.label} outputs.</p>
                  <div className="mt-3 flex flex-wrap gap-2">
                    <button
                      onClick={async () => {
                        const layout = defaultLayoutForRole(selectedRole);
                        const targets = outputs.filter(o => o.role === selectedRole);
                        await Promise.all(targets.map(o => setOutputLayout(o.id, layout)));
                        setStatus(`Applied ${layout} to ${targets.length} output(s)`);
                        await refresh();
                      }}
                      className="rounded-lg border border-accent bg-accent/10 px-3 py-2 text-xs font-bold text-accent"
                    >
                      Apply {defaultLayoutForRole(selectedRole).replace('_', ' ')}
                    </button>
                    <button
                      onClick={async () => {
                        const targets = outputs.filter(o => o.role === selectedRole);
                        await Promise.all(targets.map(o => setOutputLayout(o.id, 'auto')));
                        setStatus(`Reset ${targets.length} output(s) to auto layout`);
                        await refresh();
                      }}
                      className="rounded-lg border border-bdr px-3 py-2 text-xs font-bold text-ink-faint"
                    >
                      Reset to auto
                    </button>
                  </div>
                </div>
              )}
              <div className="grid gap-4 p-4 md:grid-cols-[1fr_240px]">
                <div>
                  <p className="mb-2 text-2xs font-bold uppercase tracking-wider text-ink-faint">Layer stack</p>
                  <div className="space-y-2">
                    {selectedRoleInfo.stack.map((layer, index) => (
                      <div key={layer} className="flex items-center gap-3 rounded-lg border border-bdr bg-surface-900 px-3 py-2">
                        <span className="flex size-6 items-center justify-center rounded bg-surface-700 text-2xs font-black text-ink-faint">{index + 1}</span>
                        <span className="text-sm font-semibold text-ink">{layer}</span>
                        {layer === 'Scripture' && <span className="ml-auto rounded bg-accent/15 px-2 py-0.5 text-2xs font-black text-accent">PRIORITY</span>}
                      </div>
                    ))}
                  </div>
                </div>
                <div className="rounded-xl border border-bdr bg-surface-900 p-3">
                  <div className="mb-2 flex items-center gap-2 text-2xs font-bold uppercase tracking-wider text-ink-faint">
                    <Route size={12} /> Routing
                  </div>
                  <p className="text-sm text-ink-muted">{selectedRoleInfo.route}</p>
                  <div className="mt-3 space-y-2">
                    <button
                      onClick={async () => {
                        const mode: ScriptureMode = scriptureMode === 'replace' ? 'overlay' : 'replace';
                        await setScriptureMode(mode);
                        setScriptureModeState(mode);
                        setStatus(`Scripture mode: ${mode}`);
                      }}
                      className="w-full rounded-lg border border-bdr bg-surface-800 px-3 py-2 text-left text-xs font-bold text-ink"
                    >
                      Scripture: <span className="text-accent">{scriptureMode}</span>
                    </button>
                    <p className="text-2xs leading-relaxed text-ink-faint">
                      Per-output role and source routing is live. Program gets the full composite; lobby, stage, and confidence follow role defaults.
                    </p>
                  </div>
                </div>
              </div>
            </section>

            <section className="inset-panel overflow-hidden">
              <div className="flex items-center justify-between border-b border-bdr px-4 py-3">
                <div>
                  <p className="text-xs font-bold uppercase tracking-wider text-ink">Physical monitors</p>
                  <p className="mt-0.5 text-2xs text-ink-faint">{monitors.length} detected</p>
                </div>
              </div>
              <div className="grid grid-cols-2 gap-3 p-4 xl:grid-cols-3">
                {monitors.map(monitor => {
                  const assignedId = assignedMonitors.get(monitor.index);
                  return (
                    <div key={monitor.index} className={['rounded-xl border p-3 transition-all', assignedId ? 'border-accent bg-accent/5' : 'border-bdr bg-surface-800'].join(' ')}>
                      <div className="relative mb-3 overflow-hidden rounded-lg border border-bdr bg-black" style={{ aspectRatio: '16/9' }}>
                        <div className="absolute inset-0 flex items-center justify-center text-2xs font-black tracking-widest text-ink-faint">
                          {assignedId ? 'CONNECTED' : 'AVAILABLE'}
                        </div>
                      </div>
                      <div className="mb-3">
                        <div className="flex items-center gap-2">
                          {monitor.is_primary && <span className="rounded bg-accent/15 px-1.5 py-0.5 text-2xs font-black text-accent">PRIMARY</span>}
                          <p className="truncate text-xs font-bold text-ink">{monitor.name}</p>
                        </div>
                        <p className="mt-0.5 text-2xs text-ink-faint">{monitor.width}x{monitor.height}</p>
                      </div>
                      {assignedId ? (
                        <button
                          onClick={() => removeOutput(assignedId).then(refresh)}
                          className="w-full rounded-md border border-danger/40 px-3 py-1.5 text-2xs font-bold text-danger transition-all hover:bg-danger hover:text-white"
                        >
                          Close output
                        </button>
                      ) : (
                        <button
                          onClick={() => openDisplay(monitor)}
                          className="w-full rounded-md border border-bdr px-3 py-1.5 text-2xs font-bold text-ink-faint transition-all hover:border-accent hover:bg-accent hover:text-surface-950"
                        >
                          Assign as {selectedRoleInfo.label}
                        </button>
                      )}
                    </div>
                  );
                })}
              </div>
            </section>

            <section className="inset-panel overflow-hidden">
              <div className="flex items-center justify-between border-b border-bdr px-4 py-3">
                <div>
                  <p className="text-xs font-bold uppercase tracking-wider text-ink">NDI outputs</p>
                  <p className="mt-0.5 text-2xs text-ink-faint">Publish any output role as an NDI feed.</p>
                </div>
                <button
                  onClick={() => setShowNdiForm(value => !value)}
                  className="flex items-center gap-1 rounded-md border border-bdr px-3 py-1.5 text-2xs font-bold text-ink-faint transition-all hover:border-accent hover:text-accent"
                >
                  <Plus size={11} /> {showNdiForm ? 'Cancel' : 'Add NDI'}
                </button>
              </div>
              {showNdiForm && (
                <div className="flex gap-2 border-b border-bdr bg-surface-800 p-3">
                  <input value={ndiLabel} onChange={e => setNdiLabel(e.target.value)} className="flex-1 rounded-md border border-bdr bg-surface-700 px-3 py-1.5 text-xs text-ink outline-none focus:border-accent" />
                  <input value={ndiSource} onChange={e => setNdiSource(e.target.value)} className="flex-1 rounded-md border border-bdr bg-surface-700 px-3 py-1.5 text-xs text-ink outline-none focus:border-accent" />
                  <button onClick={startNdi} className="rounded-md bg-accent px-4 py-1.5 text-xs font-black text-surface-950 hover:brightness-110">Start</button>
                </div>
              )}
              <div className="divide-y divide-bdr/50">
                {outputs.filter(output => output.kind.type === 'ndi').map(output => (
                  <div key={output.id} className="flex items-center gap-3 px-4 py-3">
                    <span className={['size-2 rounded-full', output.enabled ? 'bg-live shadow-[0_0_6px_theme(colors.live)]' : 'bg-ink-faint'].join(' ')} />
                    <div className="min-w-0 flex-1">
                      <p className="truncate text-sm font-bold text-ink">{output.label}</p>
                      <p className="truncate text-2xs text-ink-faint">{kindLabel(output)}</p>
                    </div>
                    <button onClick={() => toggleOutput(output.id).then(refresh)} className="rounded border border-bdr px-3 py-1 text-2xs font-black text-ink-faint hover:border-accent hover:text-accent">
                      {output.enabled ? 'ON' : 'OFF'}
                    </button>
                    <button onClick={() => removeOutput(output.id).then(refresh)} className="rounded p-1.5 text-ink-faint hover:bg-danger/10 hover:text-danger">
                      <X size={13} />
                    </button>
                  </div>
                ))}
                {outputs.filter(output => output.kind.type === 'ndi').length === 0 && (
                  <div className="px-4 py-6 text-center text-sm text-ink-faint">
                    No NDI outputs yet. Add one for OBS, vMix, switchers, or recording systems.
                  </div>
                )}
              </div>
            </section>

            <section className="inset-panel overflow-hidden">
              <div className="border-b border-bdr px-4 py-3">
                <p className="text-xs font-bold uppercase tracking-wider text-ink">Routing matrix</p>
                <p className="mt-0.5 text-2xs text-ink-faint">Assign role and source per output.</p>
              </div>
              <div className="divide-y divide-bdr/50">
                {outputs.map(output => (
                  <div key={output.id} className="grid gap-2 px-4 py-3 md:grid-cols-[1fr_130px_130px_150px_auto] md:items-center">
                    <div>
                      <p className="text-sm font-bold text-ink">{output.label}</p>
                      <p className="text-2xs text-ink-faint">{kindLabel(output)}</p>
                    </div>
                    <select
                      value={output.role}
                      onChange={e => setOutputRole(output.id, e.target.value as OutputRole).then(refresh)}
                      className="rounded-md border border-bdr bg-surface-800 px-2 py-1.5 text-xs text-ink"
                    >
                      {OUTPUT_ROLES.map(role => <option key={role.id} value={role.id}>{role.label}</option>)}
                    </select>
                    <select
                      value={output.source}
                      onChange={e => setOutputSource(output.id, e.target.value as OutputSource).then(refresh)}
                      className="rounded-md border border-bdr bg-surface-800 px-2 py-1.5 text-xs text-ink"
                    >
                      {(['auto', 'presentation', 'scripture', 'media', 'countdown'] as const).map(source => (
                        <option key={source} value={source}>{source}</option>
                      ))}
                    </select>
                    <select
                      value={output.layout ?? 'auto'}
                      onChange={e => setOutputLayout(output.id, e.target.value as RoleLayout).then(refresh)}
                      className="rounded-md border border-bdr bg-surface-800 px-2 py-1.5 text-xs text-ink"
                    >
                      {ROLE_LAYOUTS.map(layout => (
                        <option key={layout.id} value={layout.id}>{layout.label}</option>
                      ))}
                    </select>
                    <button onClick={() => toggleOutput(output.id).then(refresh)} className="rounded border border-bdr px-3 py-1 text-2xs font-black text-ink-faint">
                      {output.enabled ? 'ON' : 'OFF'}
                    </button>
                  </div>
                ))}
                {outputs.length === 0 && (
                  <p className="px-4 py-6 text-center text-sm text-ink-faint">No outputs configured yet.</p>
                )}
              </div>
            </section>
          </div>
        </section>
      </div>
    </div>
  );
}
