import { useState, useEffect } from 'react';
import { getLicenseStatus, deactivateLicense, activateLicense } from '../lib/commands';
import type { LicenseStatus } from '../lib/types';

export default function Settings() {
  const [license, setLicense] = useState<LicenseStatus | null>(null);
  const [tokenInput, setTokenInput] = useState('');
  const [activating, setActivating] = useState(false);
  const [msg, setMsg] = useState('');

  useEffect(() => {
    getLicenseStatus().then(setLicense).catch(() => {});
  }, []);

  const handleActivate = async () => {
    if (!tokenInput.trim()) return;
    setActivating(true);
    try {
      const s = await activateLicense(tokenInput.trim());
      setLicense(s);
      setTokenInput('');
      setMsg('License activated successfully.');
    } catch (e: any) {
      setMsg(e?.toString() ?? 'Activation failed');
    } finally {
      setActivating(false);
    }
  };

  const handleDeactivate = async () => {
    await deactivateLicense();
    setLicense(prev => prev ? { ...prev, plan: 'free', is_active: false } : null);
    setMsg('License removed from this device.');
  };

  const planColor = license?.plan === 'premium' ? 'text-accent' : license?.plan === 'standard' ? 'text-blue-400' : 'text-ink-faint';
  const planLabel = license?.plan === 'premium' ? 'Premium' : license?.plan === 'standard' ? 'Standard' : 'Free';

  return (
    <div className="h-full overflow-y-auto bg-surface-900 p-6">
      <div className="max-w-2xl mx-auto space-y-6">
        <div>
          <h1 className="text-lg font-bold text-ink">Settings</h1>
          <p className="text-sm text-ink-faint mt-0.5">Application preferences and licensing</p>
        </div>

        {msg && (
          <div className="px-4 py-2.5 bg-accent/10 border border-accent/30 rounded-lg text-sm text-accent">
            {msg}
          </div>
        )}

        {/* License */}
        <section className="inset-panel overflow-hidden">
          <div className="px-4 py-3 border-b border-bdr">
            <span className="text-xs font-bold text-ink uppercase tracking-wider">License</span>
          </div>
          <div className="p-4 space-y-4">
            {license && (
              <div className="flex items-center gap-4 p-3 bg-surface-800 rounded-lg">
                <div className="w-10 h-10 rounded-full bg-surface-700 border border-bdr flex items-center justify-center">
                  <span className={`text-lg font-black ${planColor}`}>{planLabel[0]}</span>
                </div>
                <div className="flex-1">
                  <div className={`text-sm font-bold ${planColor}`}>{planLabel} Plan</div>
                  {license.org && <div className="text-xs text-ink-faint">{license.org}</div>}
                  {license.expires_at && (
                    <div className="text-2xs text-ink-faint">
                      Expires {new Date(license.expires_at * 1000).toLocaleDateString()}
                      {license.is_in_grace && <span className="text-amber-400 ml-2">Grace period</span>}
                    </div>
                  )}
                </div>
                {license.plan !== 'free' && (
                  <button
                    onClick={handleDeactivate}
                    className="px-3 py-1 border border-bdr rounded text-xs text-ink-faint hover:border-danger hover:text-danger transition-colors"
                  >
                    Remove
                  </button>
                )}
              </div>
            )}

            {(!license || license.plan === 'free') && (
              <div className="space-y-2">
                <label className="text-xs font-semibold text-ink-muted block">Activate License</label>
                <div className="flex gap-2">
                  <input
                    value={tokenInput}
                    onChange={e => setTokenInput(e.target.value)}
                    placeholder="Paste your license token…"
                    className="flex-1 bg-surface-700 border border-bdr rounded-md px-3 py-2 text-xs text-ink placeholder:text-ink-faint focus:outline-none focus:border-accent font-mono"
                  />
                  <button
                    onClick={handleActivate}
                    disabled={activating || !tokenInput.trim()}
                    className="px-4 py-2 bg-accent text-surface-950 rounded-md text-xs font-bold hover:brightness-110 disabled:opacity-50 transition-all"
                  >
                    {activating ? 'Activating…' : 'Activate'}
                  </button>
                </div>
              </div>
            )}
          </div>
        </section>

        {/* Plan comparison */}
        <section className="grid grid-cols-3 gap-3">
          {([
            { planId: 'free' as const,     label: 'Free',     price: '$0',     features: ['Bible projection', 'NDI output', 'Offline first', 'Basic themes'] },
            { planId: 'standard' as const, label: 'Standard', price: '$9/mo',  features: ['Advanced templates', 'Canvas designer', '4K output', '+ Free features'] },
            { planId: 'premium' as const,  label: 'Premium',  price: '$19/mo', features: ['AI suggestions', 'Cloud sync', 'Multiple outputs', '+ Standard features'] },
          ]).map(({ planId, label, price, features }) => (
            <div key={planId} className={[
              'p-4 rounded-xl border',
              planId === license?.plan
                ? 'border-accent bg-accent/5'
                : 'border-bdr bg-surface-800',
            ].join(' ')}>
              <div className="text-sm font-bold text-ink mb-0.5">{label}</div>
              <div className="text-lg font-black text-accent mb-3">{price}</div>
              <ul className="space-y-1.5">
                {features.map(f => (
                  <li key={f} className="text-2xs text-ink-faint flex items-center gap-1.5">
                    <span className="w-1 h-1 rounded-full bg-ink-faint shrink-0" />
                    {f}
                  </li>
                ))}
              </ul>
              {planId !== license?.plan && planId !== 'free' && (
                <button className="mt-3 w-full py-1.5 bg-accent text-surface-950 rounded-md text-2xs font-bold hover:brightness-110 transition-all">
                  Upgrade
                </button>
              )}
            </div>
          ))}
        </section>

        {/* Device ID */}
        {license?.device_id && (
          <div className="text-2xs text-ink-faint font-mono px-1">
            Device ID: {license.device_id.slice(0, 16)}…
          </div>
        )}
      </div>
    </div>
  );
}
