/**
 * Isle Production Web Audio Engine:
 * - Real Web Audio FFT AnalyserNode with 64 / 128 bins
 * - 3-bar compact pill equalizer & 12-bar expanded studio spectrum visualizer
 * - Local audio file playback support (.mp3 / .wav) with real-time FFT
 * - Ambient pad oscillator synthesis synced to track chords & BPM
 * - Procedural tactile SFX (subtle click, crisp pop, water droplet, Tibetan singing bowl)
 */

class IsleAudioEngine {
  private ctx: AudioContext | null = null;
  private masterGain: GainNode | null = null;
  private analyser: AnalyserNode | null = null;
  private freqData: Uint8Array<ArrayBuffer> | null = null;
  private activeNodes: AudioNode[] = [];
  private pulseTimer: number | null = null;
  private isPlaying = false;
  private volume = 0.45;

  // Local user audio element source
  private userAudioEl: HTMLAudioElement | null = null;
  private userMediaSource: MediaElementAudioSourceNode | null = null;
  private isUserAudio = false;

  public ensureContext(): AudioContext | null {
    if (typeof window === 'undefined') return null;
    if (!this.ctx) {
      const AudioCtx =
        window.AudioContext ||
        (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
      if (!AudioCtx) return null;

      this.ctx = new AudioCtx();
      this.masterGain = this.ctx.createGain();
      this.masterGain.gain.value = this.volume * 0.18;

      this.analyser = this.ctx.createAnalyser();
      this.analyser.fftSize = 128; // 64 frequency bins
      this.analyser.smoothingTimeConstant = 0.8;
      this.freqData = new Uint8Array(this.analyser.frequencyBinCount);

      this.masterGain.connect(this.analyser);
      this.analyser.connect(this.ctx.destination);
    }
    if (this.ctx.state === 'suspended') {
      void this.ctx.resume().catch(() => {});
    }
    return this.ctx;
  }

  public getAnalyser(): AnalyserNode | null {
    this.ensureContext();
    return this.analyser;
  }

  public setVolume(vol: number): void {
    this.volume = Math.max(0, Math.min(1, vol));
    if (this.ctx && this.masterGain) {
      this.masterGain.gain.setTargetAtTime(this.volume * 0.18, this.ctx.currentTime, 0.04);
    }
    if (this.userAudioEl) {
      this.userAudioEl.volume = this.volume;
    }
  }

  /**
   * Connect a local user-loaded audio file (.mp3 / .wav) for real FFT analysis.
   */
  public playUserAudio(audioEl: HTMLAudioElement): void {
    const ctx = this.ensureContext();
    if (!ctx || !this.masterGain) return;

    this.stopAmbient();
    this.isUserAudio = true;
    this.userAudioEl = audioEl;

    if (!this.userMediaSource) {
      try {
        this.userMediaSource = ctx.createMediaElementSource(audioEl);
        this.userMediaSource.connect(this.masterGain);
      } catch {
        // Source might already be connected
      }
    }

    this.isPlaying = true;
    void audioEl.play().catch(() => {});
  }

  public pauseUserAudio(): void {
    if (this.userAudioEl) {
      this.userAudioEl.pause();
    }
    this.isPlaying = false;
  }

  public seekUserAudio(sec: number): void {
    if (this.userAudioEl) {
      this.userAudioEl.currentTime = sec;
    }
  }

  /**
   * Start procedural warm synth pad when playing built-in tracks.
   */
  public startAmbient(bpm: number, chordFreqs: number[], volume: number): void {
    if (this.isUserAudio && this.userAudioEl && !this.userAudioEl.paused) {
      return;
    }
    this.stopAmbient();
    this.volume = volume;
    const ctx = this.ensureContext();
    if (!ctx || !this.masterGain) return;

    this.isPlaying = true;
    this.masterGain.gain.setTargetAtTime(this.volume * 0.18, ctx.currentTime, 0.08);

    const filter = ctx.createBiquadFilter();
    filter.type = 'lowpass';
    filter.frequency.value = 750;
    filter.Q.value = 1.3;
    filter.connect(this.masterGain);
    this.activeNodes.push(filter);

    // Warm detuned pad voices
    chordFreqs.forEach((freq, idx) => {
      const osc = ctx.createOscillator();
      const voiceGain = ctx.createGain();
      osc.type = idx % 2 === 0 ? 'sine' : 'triangle';
      osc.frequency.value = freq * 0.5;
      osc.detune.value = (idx - 1.5) * 5;
      voiceGain.gain.value = 0.16 / chordFreqs.length;
      osc.connect(voiceGain);
      voiceGain.connect(filter);
      osc.start();
      this.activeNodes.push(osc, voiceGain);
    });

    // Gentle rhythmic pulse synced to BPM
    const beatMs = Math.max(260, Math.round((60 / Math.max(60, bpm)) * 1000));
    let step = 0;
    const triggerPulse = () => {
      if (!this.isPlaying || !this.ctx || !this.masterGain) return;
      const now = this.ctx.currentTime;
      const osc = this.ctx.createOscillator();
      const g = this.ctx.createGain();
      const note = chordFreqs[step % chordFreqs.length];
      step += 1;

      osc.type = 'sine';
      osc.frequency.setValueAtTime(note, now);
      g.gain.setValueAtTime(0.001, now);
      g.gain.exponentialRampToValueAtTime(0.14, now + 0.03);
      g.gain.exponentialRampToValueAtTime(0.001, now + 0.36);

      osc.connect(g);
      g.connect(this.masterGain);
      osc.start(now);
      osc.stop(now + 0.38);
    };

    triggerPulse();
    this.pulseTimer = window.setInterval(triggerPulse, beatMs);
  }

  public stopAmbient(): void {
    this.isPlaying = false;
    if (this.pulseTimer !== null) {
      window.clearInterval(this.pulseTimer);
      this.pulseTimer = null;
    }
    for (const node of this.activeNodes) {
      try {
        if ('stop' in node && typeof (node as OscillatorNode).stop === 'function') {
          (node as OscillatorNode).stop();
        }
        node.disconnect();
      } catch {
        // Ignore already-stopped nodes
      }
    }
    this.activeNodes = [];
  }

  /**
   * Sample 3 normalized energy bands (low, mid, high) for compact pill equalizer.
   */
  public sampleBands(timeSec: number, bpm: number): [number, number, number] {
    if (this.analyser && this.freqData && this.isPlaying && this.volume > 0.02) {
      this.analyser.getByteFrequencyData(this.freqData);
      const b0 = (this.freqData[1] + this.freqData[2] + this.freqData[3]) / 765;
      const b1 = (this.freqData[6] + this.freqData[8] + this.freqData[10]) / 765;
      const b2 = (this.freqData[14] + this.freqData[18] + this.freqData[24]) / 765;
      const sum = b0 + b1 + b2;
      if (sum > 0.04) {
        return [
          Math.min(1, Math.max(0.2, b0 * 1.9)),
          Math.min(1, Math.max(0.24, b1 * 2.2)),
          Math.min(1, Math.max(0.18, b2 * 2.5)),
        ];
      }
    }

    // BPM-locked procedural fallback
    const beat = (timeSec * bpm) / 60;
    return [
      0.35 + 0.55 * Math.abs(Math.sin(beat * Math.PI)),
      0.3 + 0.65 * Math.abs(Math.sin(beat * Math.PI * 1.35 + 0.9)),
      0.28 + 0.58 * Math.abs(Math.cos(beat * Math.PI * 0.85 + 1.7)),
    ];
  }

  /**
   * Sample 12 real FFT frequency bins (0..1) for the expanded studio visualizer.
   */
  public sample12Bands(timeSec: number, bpm: number): number[] {
    if (this.analyser && this.freqData && this.isPlaying && this.volume > 0.02) {
      this.analyser.getByteFrequencyData(this.freqData);
      const bins: number[] = [];
      const binIndices = [1, 2, 4, 6, 8, 11, 15, 20, 26, 33, 42, 52];
      for (const idx of binIndices) {
        const val = (this.freqData[idx] ?? 0) / 255;
        bins.push(Math.min(1, Math.max(0.12, val * 1.7)));
      }
      return bins;
    }

    // Smooth procedural 12-bar fallback
    const beat = (timeSec * bpm) / 60;
    const fallback: number[] = [];
    for (let i = 0; i < 12; i++) {
      const v = 0.2 + 0.7 * Math.abs(Math.sin(beat * Math.PI * (0.8 + i * 0.12) + i * 0.4));
      fallback.push(Number(v.toFixed(2)));
    }
    return fallback;
  }

  // ---------------------------------------------------------------------------
  // Procedural Web Audio SFX (Zero External Audio Asset Files)
  // ---------------------------------------------------------------------------

  /**
   * Subtle click on tab slide or volume scroll.
   */
  public playClick(): void {
    const ctx = this.ensureContext();
    if (!ctx) return;
    const now = ctx.currentTime;
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();

    osc.type = 'sine';
    osc.frequency.setValueAtTime(1400, now);
    osc.frequency.exponentialRampToValueAtTime(450, now + 0.02);

    gain.gain.setValueAtTime(0.04, now);
    gain.gain.exponentialRampToValueAtTime(0.0001, now + 0.025);

    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.start(now);
    osc.stop(now + 0.03);
  }

  /**
   * Crisp pop on Island expand / collapse.
   */
  public playPop(): void {
    const ctx = this.ensureContext();
    if (!ctx) return;
    const now = ctx.currentTime;
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();

    osc.type = 'sine';
    osc.frequency.setValueAtTime(460, now);
    osc.frequency.exponentialRampToValueAtTime(180, now + 0.045);

    gain.gain.setValueAtTime(0.08, now);
    gain.gain.exponentialRampToValueAtTime(0.0001, now + 0.05);

    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.start(now);
    osc.stop(now + 0.055);
  }

  /**
   * Water droplet sound on +250ml log.
   */
  public playWaterDroplet(): void {
    const ctx = this.ensureContext();
    if (!ctx) return;
    const now = ctx.currentTime;

    // Resonant water drop pitch upward sweep
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    const filter = ctx.createBiquadFilter();

    filter.type = 'bandpass';
    filter.frequency.setValueAtTime(1200, now);
    filter.Q.value = 3.5;

    osc.type = 'sine';
    osc.frequency.setValueAtTime(520, now);
    osc.frequency.exponentialRampToValueAtTime(1750, now + 0.1);
    osc.frequency.exponentialRampToValueAtTime(1380, now + 0.16);

    gain.gain.setValueAtTime(0.001, now);
    gain.gain.linearRampToValueAtTime(0.12, now + 0.02);
    gain.gain.exponentialRampToValueAtTime(0.0001, now + 0.2);

    osc.connect(filter);
    filter.connect(gain);
    gain.connect(ctx.destination);

    osc.start(now);
    osc.stop(now + 0.22);
  }

  /**
   * Calming Tibetan singing bowl chime on Pomodoro completion.
   */
  public playSingingBowl(): void {
    const ctx = this.ensureContext();
    if (!ctx) return;
    const now = ctx.currentTime;

    // Harmonics of Tibetan singing bowl (432Hz fundamental + partials)
    const partials = [
      { freq: 432, gain: 0.16, decay: 3.2 },
      { freq: 864, gain: 0.09, decay: 2.6 },
      { freq: 1296, gain: 0.06, decay: 1.8 },
      { freq: 1840, gain: 0.035, decay: 1.4 },
      { freq: 2420, gain: 0.02, decay: 0.9 },
    ];

    partials.forEach(({ freq, gain, decay }) => {
      const osc = ctx.createOscillator();
      const g = ctx.createGain();

      osc.type = 'sine';
      osc.frequency.setValueAtTime(freq, now);

      g.gain.setValueAtTime(0.0001, now);
      g.gain.linearRampToValueAtTime(gain, now + 0.04);
      g.gain.exponentialRampToValueAtTime(0.0001, now + decay);

      osc.connect(g);
      g.connect(ctx.destination);

      osc.start(now);
      osc.stop(now + decay + 0.1);
    });
  }

  // Aliases for compatibility
  public playChime(): void {
    this.playSingingBowl();
  }

  public playSplash(): void {
    this.playWaterDroplet();
  }

  public playTick(): void {
    this.playClick();
  }
}

export const audioEngine = new IsleAudioEngine();
