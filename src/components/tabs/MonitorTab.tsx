import React, { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { ArrowDown, ArrowUp, Cpu, Gauge, HardDrive } from 'lucide-react';

interface TelemetrySnapshot {
  cpuPct: number;
  cores: number[];
  ramUsedGb: number;
  ramTotalGb: number;
  gpuPct: number;
  gpuTempC: number;
  downMbps: number;
  upMbps: number;
}

function clamp(val: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, val));
}

export const MonitorTab: React.FC = () => {
  const [stats, setStats] = useState<TelemetrySnapshot>({
    cpuPct: 34,
    cores: [42, 28, 56, 19, 38, 44, 22, 31],
    ramUsedGb: 11.8,
    ramTotalGb: 16.0,
    gpuPct: 41,
    gpuTempC: 52,
    downMbps: 12.4,
    upMbps: 1.8,
  });

  useEffect(() => {
    const id = window.setInterval(() => {
      setStats((prev) => {
        const nextCores = prev.cores.map((c) =>
          Math.round(clamp(c + (Math.random() - 0.48) * 18, 8, 94))
        );
        const avgCpu = Math.round(
          nextCores.reduce((acc, v) => acc + v, 0) / nextCores.length
        );
        const nextRam = Number(
          clamp(prev.ramUsedGb + (Math.random() - 0.5) * 0.2, 9.4, 14.8).toFixed(1)
        );
        const nextGpu = Math.round(clamp(prev.gpuPct + (Math.random() - 0.5) * 10, 14, 88));
        const nextTemp = Math.round(clamp(46 + nextGpu * 0.18, 44, 74));
        const nextDown = Number(
          clamp(prev.downMbps + (Math.random() - 0.48) * 3.4, 1.2, 42.0).toFixed(1)
        );
        const nextUp = Number(
          clamp(prev.upMbps + (Math.random() - 0.5) * 0.8, 0.3, 9.5).toFixed(1)
        );
        return {
          cpuPct: avgCpu,
          cores: nextCores,
          ramUsedGb: nextRam,
          ramTotalGb: prev.ramTotalGb,
          gpuPct: nextGpu,
          gpuTempC: nextTemp,
          downMbps: nextDown,
          upMbps: nextUp,
        };
      });
    }, 1100);
    return () => window.clearInterval(id);
  }, []);

  const ramPct = Math.round((stats.ramUsedGb / stats.ramTotalGb) * 100);

  return (
    <div className="w-full h-full px-4 pb-3 flex flex-col justify-between gap-2 select-none">
      {/* Top Grid: CPU Multi-Core + RAM & GPU */}
      <div className="grid grid-cols-2 gap-2.5 flex-1">
        {/* CPU Card */}
        <div className="rounded-2xl bg-white/[0.04] border border-white/[0.08] p-2.5 flex flex-col justify-between">
          <div className="flex items-center justify-between">
            <span className="inline-flex items-center gap-1.5 text-[10.5px] font-medium text-white/65">
              <Cpu className="w-3.5 h-3.5 text-emerald-400" />
              CPU Load
            </span>
            <span className="text-[14px] font-bold text-white tabular-nums">
              {stats.cpuPct}%
            </span>
          </div>

          {/* 8-Core Thread Activity Bars */}
          <div className="flex items-end justify-between gap-1.5 h-14 pt-2">
            {stats.cores.map((load, idx) => (
              <div
                key={idx}
                className="flex-1 h-full rounded-md bg-white/[0.06] overflow-hidden flex items-end"
                title={`Core ${idx + 1}: ${load}%`}
              >
                <motion.div
                  className="w-full rounded-md"
                  style={{
                    backgroundColor:
                      load > 75 ? '#F87171' : load > 50 ? '#FBBF24' : '#34D399',
                  }}
                  animate={{ height: `${load}%` }}
                  transition={{ type: 'spring', stiffness: 260, damping: 24 }}
                />
              </div>
            ))}
          </div>

          <div className="flex items-center justify-between text-[9px] text-white/40 mt-1">
            <span>8 Threads Active</span>
            <span>4.2 GHz</span>
          </div>
        </div>

        {/* RAM + GPU Stack */}
        <div className="flex flex-col gap-2">
          {/* RAM Usage */}
          <div className="flex-1 rounded-2xl bg-white/[0.04] border border-white/[0.08] px-3 py-2 flex flex-col justify-between">
            <div className="flex items-center justify-between">
              <span className="inline-flex items-center gap-1.5 text-[10.5px] font-medium text-white/65">
                <HardDrive className="w-3.5 h-3.5 text-sky-400" />
                RAM
              </span>
              <span className="text-[11px] font-semibold text-white tabular-nums">
                {stats.ramUsedGb.toFixed(1)} / {stats.ramTotalGb.toFixed(1)} GB · {ramPct}%
              </span>
            </div>
            <div className="w-full h-1.5 rounded-full bg-white/10 overflow-hidden mt-1">
              <motion.div
                className="h-full rounded-full bg-sky-400"
                animate={{ width: `${ramPct}%` }}
                transition={{ duration: 0.4 }}
              />
            </div>
          </div>

          {/* GPU Load & Temp */}
          <div className="flex-1 rounded-2xl bg-white/[0.04] border border-white/[0.08] px-3 py-2 flex flex-col justify-between">
            <div className="flex items-center justify-between">
              <span className="inline-flex items-center gap-1.5 text-[10.5px] font-medium text-white/65">
                <Gauge className="w-3.5 h-3.5 text-amber-400" />
                GPU
              </span>
              <span className="text-[11px] font-semibold text-white tabular-nums">
                {stats.gpuPct}% · <span className="text-amber-300">{stats.gpuTempC}°C</span>
              </span>
            </div>
            <div className="w-full h-1.5 rounded-full bg-white/10 overflow-hidden mt-1">
              <motion.div
                className="h-full rounded-full bg-amber-400"
                animate={{ width: `${stats.gpuPct}%` }}
                transition={{ duration: 0.4 }}
              />
            </div>
          </div>
        </div>
      </div>

      {/* Bottom Live Network Speed Ticker */}
      <div className="h-8 rounded-xl bg-white/[0.04] border border-white/[0.08] px-3 flex items-center justify-between text-[11px]">
        <span className="text-white/50 font-medium">Live Network</span>
        <div className="flex items-center gap-4 tabular-nums font-semibold">
          <span className="inline-flex items-center gap-1 text-emerald-300">
            <ArrowDown className="w-3 h-3" />
            {stats.downMbps.toFixed(1)} MB/s
          </span>
          <span className="inline-flex items-center gap-1 text-sky-300">
            <ArrowUp className="w-3 h-3" />
            {stats.upMbps.toFixed(1)} MB/s
          </span>
        </div>
      </div>
    </div>
  );
};
