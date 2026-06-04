import { useState, useEffect } from 'react';
import { RefreshCw, Plus, X } from 'lucide-react';
import { listMonitors, getOutputs, addNdiOutput, addDisplayOutput, removeOutput, toggleOutput } from '../lib/commands';
import type { MonitorInfo, OutputInfo } from '../lib/types';

export default function Outputs() {
  const [monitors, setMonitors] = useState<MonitorInfo[]>([]);
  const [outputs, setOutputs] = useState<OutputInfo[]>([]);
  const [status, setStatus] = useState('');

  const [showNdiForm, setShowNdiForm] = useState(false);
  const [ndiLabel, setNdiLabel] = useState('NDI Output');
  const [ndiSource, setNdiSource] = useState('BiblePro');

  const refresh = async () => {
    const [mons, outs] = await Promise.all([listMonitors(), getOutputs()]);
    setMonitors(mons);
    setOutputs(outs);
  };

  useEffect(() => { refresh(); }, []);

  const assignedMonitors = new Map(
    outputs
      .filter(o => o.kind.type === 'display')
      .map(o => {
        const mi = (o.kind as any).monitor_index as number;
        return [mi, o.id];
      })
  );

  const openDisplay = async (m: MonitorInfo) => {
    const label = m.is_primary ? `Display ${m.index + 1} (Primary)` : `Display ${m.index + 1}`;
    try {
      await addDisplayOutput(label, m.index, m.name, m.x, m.y, m.width, m.height);
      setStatus(`Opened display on ${m.name}`);
      await refresh();
    } catch (e: any) { setStatus(e?.toString() ?? 'Error'); }
  };

  const closeDisplay = async (id: string) => {
    await removeOutput(id);
    setStatus('Display closed');
    await refresh();
  };

  const startNdi = async () => {
    try {
      await addNdiOutput(ndiLabel, ndiSource);
      setShowNdiForm(false);
      setStatus(`NDI '${ndiSource}' started`);
      await refresh();
    } catch (e: any) { setStatus(e?.toString() ?? 'Error'); }
  };

  const ndiOutputs = outputs.filter(o => o.kind.type === 'ndi');

  return (
    <div className="h-full overflow-y-auto bg-surface-900 p-6">
      <div className="max-w-3xl mx-auto space-y-6">

        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-lg font-bold text-ink">Outputs</h1>
            <p className="text-sm text-ink-faint mt-0.5">Manage displays and NDI sources</p>
          </div>
          <button
            onClick={refresh}
            className="flex items-center gap-1.5 px-3 py-1.5 bg-surface-700 border border-bdr rounded-md text-xs text-ink-muted hover:text-ink hover:bg-surface-600 transition-colors"
          >
            <RefreshCw size={13} /> Refresh
          </button>
        </div>

        {status && (
          <div className="px-4 py-2.5 bg-accent/10 border border-accent/30 rounded-lg text-sm text-accent">
            {status}
          </div>
        )}

        {/* ── Displays ─────────────────────────────────────────────────── */}
        <section className="inset-panel overflow-hidden">
          <div className="flex items-center justify-between px-4 py-3 border-b border-bdr">
            <span className="text-xs font-bold text-ink uppercase tracking-wider">Displays</span>
            <span className="text-2xs text-ink-faint">{monitors.length} monitor{monitors.length !== 1 ? 's' : ''} detected</span>
          </div>

          {monitors.length === 0 ? (
            <div className="px-4 py-6 text-sm text-ink-faint text-center">No monitors detected.</div>
          ) : (
            <div className="grid grid-cols-3 gap-3 p-4">
              {monitors.map(m => {
                const assignedId = assignedMonitors.get(m.index);
                const isLive = !!assignedId;
                return (
                  <div
                    key={m.index}
                    className={[
                      'flex flex-col rounded-xl border transition-all duration-150 overflow-hidden',
                      isLive
                        ? 'border-accent bg-accent/5'
                        : 'border-bdr bg-surface-800',
                    ].join(' ')}
                  >
                    {/* Screen visual */}
                    <div className="relative mx-3 mt-3 rounded-lg overflow-hidden border border-bdr/60" style={{ aspectRatio: '16/9', background: '#000' }}>
                      {isLive && (
                        <div className="absolute inset-0 flex items-center justify-center">
                          <div className="text-accent text-2xs font-black tracking-widest opacity-60">LIVE</div>
                        </div>
                      )}
                      <div className="absolute top-1.5 right-1.5">
                        {isLive && (
                          <span className="px-1.5 py-0.5 bg-accent text-surface-950 rounded text-2xs font-black">ON</span>
                        )}
                      </div>
                      {/* Monitor frame base */}
                      <div className="absolute inset-0 border border-bdr/40" />
                    </div>

                    {/* Info */}
                    <div className="px-3 py-2">
                      <div className="flex items-center gap-1.5 mb-0.5">
                        {m.is_primary && (
                          <span className="text-2xs font-bold text-accent bg-accent/15 px-1.5 py-0.5 rounded">PRIMARY</span>
                        )}
                        <span className="text-xs font-semibold text-ink truncate">{m.name}</span>
                      </div>
                      <div className="text-2xs text-ink-faint">{m.width}×{m.height}</div>
                    </div>

                    {/* Action */}
                    <div className="px-3 pb-3">
                      {isLive ? (
                        <button
                          onClick={() => closeDisplay(assignedId!)}
                          className="w-full py-1.5 rounded-md text-2xs font-bold border border-danger/40 text-danger hover:bg-danger hover:text-white transition-all"
                        >
                          Close Window
                        </button>
                      ) : (
                        <button
                          onClick={() => openDisplay(m)}
                          className="w-full py-1.5 rounded-md text-2xs font-bold border border-bdr text-ink-muted hover:bg-accent hover:text-surface-950 hover:border-accent transition-all"
                        >
                          Open Display
                        </button>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </section>

        {/* ── NDI Sources ──────────────────────────────────────────────── */}
        <section className="inset-panel overflow-hidden">
          <div className="flex items-center justify-between px-4 py-3 border-b border-bdr">
            <span className="text-xs font-bold text-ink uppercase tracking-wider">NDI Sources</span>
            <button
              onClick={() => setShowNdiForm(f => !f)}
              className={[
                'flex items-center gap-1 px-3 py-1 rounded-md text-2xs font-bold border transition-all',
                showNdiForm
                  ? 'bg-surface-600 border-bdr text-ink'
                  : 'border-bdr text-ink-faint hover:text-accent hover:border-accent',
              ].join(' ')}
            >
              <Plus size={11} />
              {showNdiForm ? 'Cancel' : 'Add NDI'}
            </button>
          </div>

          {showNdiForm && (
            <div className="flex gap-2 p-3 border-b border-bdr bg-surface-800">
              <input
                value={ndiLabel}
                onChange={e => setNdiLabel(e.target.value)}
                placeholder="Label"
                className="flex-1 bg-surface-700 border border-bdr rounded-md px-3 py-1.5 text-xs text-ink placeholder:text-ink-faint focus:outline-none focus:border-accent"
              />
              <input
                value={ndiSource}
                onChange={e => setNdiSource(e.target.value)}
                placeholder="Source name"
                className="flex-1 bg-surface-700 border border-bdr rounded-md px-3 py-1.5 text-xs text-ink placeholder:text-ink-faint focus:outline-none focus:border-accent"
              />
              <button
                onClick={startNdi}
                className="px-4 py-1.5 bg-accent text-surface-950 rounded-md text-xs font-bold hover:brightness-110 transition-all"
              >
                Start
              </button>
            </div>
          )}

          {ndiOutputs.length === 0 && !showNdiForm ? (
            <div className="px-4 py-5 text-sm text-ink-faint text-center">
              No NDI sources. NDI lets other software receive your slide output over the network.
            </div>
          ) : (
            <div className="divide-y divide-bdr/50">
              {ndiOutputs.map(o => {
                const sourceName = (o.kind as any).source_name as string;
                return (
                  <div key={o.id} className="flex items-center gap-3 px-4 py-3">
                    <div className={[
                      'w-2 h-2 rounded-full shrink-0',
                      o.enabled ? 'bg-live shadow-[0_0_6px_theme(colors.live)]' : 'bg-ink-faint',
                    ].join(' ')} />
                    <div className="flex-1 min-w-0">
                      <div className="text-sm font-semibold text-ink">{o.label}</div>
                      <div className="text-2xs text-ink-faint">{sourceName}</div>
                    </div>
                    <button
                      onClick={() => toggleOutput(o.id).then(refresh)}
                      className={[
                        'px-3 py-1 rounded text-2xs font-black border transition-all',
                        o.enabled
                          ? 'bg-accent text-surface-950 border-accent'
                          : 'bg-surface-700 border-bdr text-ink-faint hover:border-bdr-strong',
                      ].join(' ')}
                    >
                      {o.enabled ? 'ON' : 'OFF'}
                    </button>
                    <button
                      onClick={() => removeOutput(o.id).then(refresh)}
                      className="p-1.5 rounded text-ink-faint hover:text-danger hover:bg-danger/10 transition-all"
                    >
                      <X size={13} />
                    </button>
                  </div>
                );
              })}
            </div>
          )}
        </section>

        {/* ── NDI Info ─────────────────────────────────────────────────── */}
        <div className="p-4 bg-surface-800 border border-bdr rounded-lg text-xs text-ink-faint leading-relaxed">
          <strong className="text-ink-muted">NDI is free.</strong> No license required for NDI output.
          Connect ProPresenter, vMix, OBS, or any NDI-compatible software to receive live slides.
        </div>
      </div>
    </div>
  );
}
