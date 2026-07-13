let audioContext = null;

function getAudioContext() {
  if (!audioContext) {
    audioContext = new (window.AudioContext || window.webkitAudioContext)();
  }
  return audioContext;
}

function autocorrelation(buffer, sampleRate) {
  let SIZE = buffer.length;
  let rms = 0;

  for (let i = 0; i < SIZE; i++) {
    const val = buffer[i];
    rms += val * val;
  }
  rms = Math.sqrt(rms / SIZE);
  if (rms < 0.01) return -1;

  let r1 = 0, r2 = SIZE - 1;
  const threshold = 0.2;
  for (let i = 0; i < SIZE / 2; i++) {
    if (Math.abs(buffer[i]) < threshold) { r1 = i; break; }
  }
  for (let i = 1; i < SIZE / 2; i++) {
    if (Math.abs(buffer[SIZE - i]) < threshold) { r2 = SIZE - i; break; }
  }

  buffer = buffer.slice(r1, r2);
  SIZE = buffer.length;

  const c = new Float32Array(SIZE);
  for (let i = 0; i < SIZE; i++) {
    for (let j = 0; j < SIZE - i; j++) {
      c[i] = c[i] + buffer[j] * buffer[j + i];
    }
  }

  let d = 0;
  while (c[d] > c[d + 1]) d++;
  let maxval = -1;
  let maxpos = -1;
  for (let i = d; i < SIZE; i++) {
    if (c[i] > maxval) {
      maxval = c[i];
      maxpos = i;
    }
  }
  let T0 = maxpos;

  // Parabolic interpolation
  const x1 = c[T0 - 1];
  const x2 = c[T0];
  const x3 = c[T0 + 1];
  const a = (x1 + x3 - 2 * x2) / 2;
  const b = (x3 - x1) / 2;
  if (a) T0 = T0 - b / (2 * a);

  return sampleRate / T0;
}

export async function detectPitch(file) {
  const ctx = getAudioContext();
  const arrayBuffer = await file.arrayBuffer();
  const audioBuffer = await ctx.decodeAudioData(arrayBuffer);

  const channelData = audioBuffer.getChannelData(0);
  const sampleRate = audioBuffer.sampleRate;

  const blockSize = 4096;
  const frequencies = [];

  for (let i = 0; i + blockSize < channelData.length; i += blockSize / 2) {
    const block = channelData.slice(i, i + blockSize);
    const freq = autocorrelation(block, sampleRate);
    if (freq > 0 && freq < 10000) {
      frequencies.push(freq);
    }
  }

  if (frequencies.length === 0) return null;

  frequencies.sort((a, b) => a - b);
  const median = frequencies[Math.floor(frequencies.length / 2)];

  const filtered = frequencies.filter(f => Math.abs(f - median) < median * 0.15);
  if (filtered.length === 0) return null;

  const avgFreq = filtered.reduce((s, f) => s + f, 0) / filtered.length;

  const midi = 69 + 12 * Math.log2(avgFreq / 440);
  return Math.round(midi);
}
