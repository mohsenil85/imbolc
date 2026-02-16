// imbolc_universe SynthDef — Korg M1 "Universe" style
// Lush choir pad + metallic FM bell shimmer + noise air wash
(
var dir = thisProcess.nowExecutingPath.dirname.dirname.dirname;

SynthDef(\imbolc_universe, { |out=1024, freq_in=(-1), gate_in=(-1), vel_in=(-1),
    freq=440, amp=0.5, lag=0.02, shimmer=0.4, warmth=0.5, air=0.2,
    attack=0.8, decay=0.2, sustain=0.8, release=1.0,
    amp_mod_in=(-1), pitch_mod_in=(-1)|

    var freqSig = Select.kr(freq_in >= 0, [freq.lag(lag), In.kr(freq_in)]);
    var gateSig = Select.kr(gate_in >= 0, [1, In.kr(gate_in)]);
    var velSig = Select.kr(vel_in >= 0, [1, In.kr(vel_in)]);
    var ampMod = Select.kr(amp_mod_in >= 0, [0, In.kr(amp_mod_in)]);
    var pitchMod = Select.kr(pitch_mod_in >= 0, [0, In.kr(pitch_mod_in)]);
    var finalFreq = freqSig * (2 ** pitchMod);
    var finalAmp = amp.lag(lag) * (1 + ampMod).max(0);
    var finalShimmer = shimmer.lag(lag);
    var finalWarmth = warmth.lag(lag);
    var finalAir = air.lag(lag);

    // Vibrato for choir bed
    var vib = SinOsc.kr(5.2) * 0.003 * finalFreq;

    // Layer 1: Choir bed — 4 detuned saws through formant BPFs
    var saws = Array.fill(4, { |i|
        var dt = 0.015 * (i - 1.5) / 1.5;
        Saw.ar((finalFreq + vib) * (1 + dt));
    });
    var sawMix = saws.sum * 0.25;
    var formantFreq = LinLin.kr(finalWarmth, 0, 1, 600, 1200);
    var choir = BPF.ar(sawMix, formantFreq, 0.25) + BPF.ar(sawMix, formantFreq * 2.5, 0.3) * 0.4;
    var choirSig = choir + (sawMix * 0.2);

    // Layer 2: FM bell shimmer — inharmonic ratio sqrt(2)
    var bellMod = SinOsc.ar(finalFreq * 1.4142, 0, finalFreq * 1.4142 * 2.5);
    var bell = SinOsc.ar(finalFreq + bellMod) * 0.3;
    var bellLfo = SinOsc.kr(0.3) * 0.5 + 0.5; // drift in/out
    var bellSig = bell * bellLfo * finalShimmer;

    // Layer 3: Air — BPF filtered white noise around 3kHz
    var airSig = BPF.ar(WhiteNoise.ar(1), 3000, 0.5) * finalAir * 0.15;

    // Combine layers
    var sig = choirSig + bellSig + airSig;

    // Envelope
    var env = EnvGen.ar(Env.adsr(attack, decay, sustain, release), gateSig, doneAction: 14);

    // Stereo chorus with different LFO rates on L/R
    var chorusL = DelayC.ar(sig, 0.05, SinOsc.kr(0.4, 0, 0.004, 0.012));
    var chorusR = DelayC.ar(sig, 0.05, SinOsc.kr(0.53, 0, 0.004, 0.012));

    Out.ar(out, [chorusL, chorusR] * finalAmp * velSig * env);
}).writeDefFile(dir);
)
