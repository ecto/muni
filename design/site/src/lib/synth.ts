// ── Mote Vocal Synth ────────────────────────────────────────
// Ported from ecto/mote src/synth.rs
// Six continuous parameters drive PCM synthesis.

const SAMPLE_RATE = 44100;

export interface VocalParams {
  volume: number;       // 0–1
  freqHz: number;       // fundamental frequency
  formantRatio: number; // harmonic overtone ratio
  brightness: number;   // 0 = sine, 1 = noise
  attack: number;       // 0 = soft 50ms, 1 = instant
  sweepOctaves: number; // freq glide in octaves
}

export interface Segment {
  params: VocalParams;
  ms: number;
  delayMs?: number;
}

/** Port of mote's synthesize_vocal — renders mono f32 PCM at 44.1kHz. */
export function synthesizeVocal(params: VocalParams, durationMs: number = 150): Float32Array {
  const durationSamples = Math.round(SAMPLE_RATE * durationMs / 1000);
  const samples = new Float32Array(durationSamples);

  let phase = 0;
  let formantPhase = 0;

  const releaseSamples = Math.round(0.050 * SAMPLE_RATE);
  const attackSamples = Math.round((1.0 - params.attack) * 0.050 * SAMPLE_RATE);

  for (let i = 0; i < durationSamples; i++) {
    const frac = i / durationSamples;

    const freq = params.freqHz * Math.pow(2, params.sweepOctaves * frac);

    const sine = Math.sin(phase * Math.PI * 2);
    const noise = Math.random() * 2 - 1;
    const base = sine * (1 - params.brightness) + noise * params.brightness;

    const formant = Math.sin(formantPhase * Math.PI * 2);
    const mixed = base * 0.7 + formant * 0.3;

    let env: number;
    if (i < attackSamples) {
      env = i / Math.max(attackSamples, 1);
    } else if (i >= durationSamples - releaseSamples) {
      env = (durationSamples - i) / releaseSamples;
    } else {
      env = 1.0;
    }

    samples[i] = mixed * env * params.volume;

    const dt = 1.0 / SAMPLE_RATE;
    phase += freq * dt;
    if (phase > 1.0) phase -= 1.0;
    formantPhase += freq * params.formantRatio * dt;
    if (formantPhase > 1.0) formantPhase -= Math.floor(formantPhase);
  }

  return samples;
}

/** Render multiple synth segments into a single buffer with timing offsets. */
export function renderSequence(segments: Segment[]): Float32Array {
  const positions: Array<{ offsetSamples: number; samples: Float32Array }> = [];
  let cursor = 0;
  for (const seg of segments) {
    const offsetMs = seg.delayMs ?? cursor;
    const offsetSamples = Math.round(SAMPLE_RATE * offsetMs / 1000);
    const rendered = synthesizeVocal(seg.params, seg.ms);
    positions.push({ offsetSamples, samples: rendered });
    cursor = offsetMs + seg.ms;
  }

  const totalSamples = Math.round(SAMPLE_RATE * cursor / 1000);
  const output = new Float32Array(totalSamples);
  for (const { offsetSamples, samples } of positions) {
    for (let i = 0; i < samples.length && offsetSamples + i < output.length; i++) {
      output[offsetSamples + i] += samples[i];
    }
  }

  return output;
}

/** Play a Float32Array buffer through Web Audio. */
export function playBuffer(ctx: AudioContext, samples: Float32Array) {
  const buffer = ctx.createBuffer(1, samples.length, SAMPLE_RATE);
  buffer.getChannelData(0).set(samples);
  const source = ctx.createBufferSource();
  source.buffer = buffer;
  source.connect(ctx.destination);
  source.start();
  return source;
}

export { SAMPLE_RATE };
