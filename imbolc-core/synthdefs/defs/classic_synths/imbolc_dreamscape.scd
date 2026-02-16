// imbolc_dreamscape SynthDef — Alesis-style Atmospheric Soundscape
// Evolving pad with stochastic bird-like chirps and random chime tones
(
var dir = thisProcess.nowExecutingPath.dirname.dirname.dirname;

SynthDef(\imbolc_dreamscape, { |out=1024, freq_in=(-1), gate_in=(-1), vel_in=(-1),
    freq=440, amp=0.5, lag=0.02, density=0.4, evolve=0.5, brightness=0.6,
    attack=1.0, decay=0.3, sustain=0.8, release=1.5,
    amp_mod_in=(-1), pitch_mod_in=(-1)|

    var freqSig = Select.kr(freq_in >= 0, [freq.lag(lag), In.kr(freq_in)]);
    var gateSig = Select.kr(gate_in >= 0, [1, In.kr(gate_in)]);
    var velSig = Select.kr(vel_in >= 0, [1, In.kr(vel_in)]);
    var ampMod = Select.kr(amp_mod_in >= 0, [0, In.kr(amp_mod_in)]);
    var pitchMod = Select.kr(pitch_mod_in >= 0, [0, In.kr(pitch_mod_in)]);
    var finalFreq = freqSig * (2 ** pitchMod);
    var finalAmp = amp.lag(lag) * (1 + ampMod).max(0);
    var finalDensity = density.lag(lag);
    var finalEvolve = evolve.lag(lag);
    var finalBrightness = brightness.lag(lag);
    var dustRate = LinLin.kr(finalDensity, 0, 1, 0.3, 5);

    // Layer 1: Pad bed — 4 detuned saws with slow filter LFO
    var saws = Array.fill(4, { |i|
        var dt = 0.012 * (i - 1.5) / 1.5;
        Saw.ar(finalFreq * (1 + dt));
    });
    var padMix = saws.sum * 0.25;
    var filterLfo = SinOsc.kr(0.08) * finalEvolve;
    var filterFreq = LinLin.kr(finalBrightness + (filterLfo * 0.3), 0, 1, 400, 6000);
    var pad = LPF.ar(padMix, filterFreq.clip(100, 12000));

    // Layer 2: Chirps — Dust-triggered noise bursts through random BPF
    var chirpTrigL = Dust.kr(dustRate * 0.7);
    var chirpTrigR = Dust.kr(dustRate * 0.6);
    var chirpFreqL = TRand.kr(2000, 8000, chirpTrigL);
    var chirpFreqR = TRand.kr(2000, 8000, chirpTrigR);
    var chirpL = BPF.ar(WhiteNoise.ar(1), chirpFreqL, 0.05) * EnvGen.ar(Env.perc(0.005, 0.08), chirpTrigL) * 0.4;
    var chirpR = BPF.ar(WhiteNoise.ar(1), chirpFreqR, 0.05) * EnvGen.ar(Env.perc(0.005, 0.08), chirpTrigR) * 0.4;

    // Layer 3: Chimes — Dust-triggered FM bell tones at random intervals
    var chimeTrigL = Dust.kr(dustRate * 0.3);
    var chimeTrigR = Dust.kr(dustRate * 0.25);
    var chimeRatioL = TRand.kr(1, 4, chimeTrigL).round(1);
    var chimeRatioR = TRand.kr(1, 4, chimeTrigR).round(1);
    var chimeFreqL = finalFreq * chimeRatioL;
    var chimeFreqR = finalFreq * chimeRatioR;
    var chimeModL = SinOsc.ar(chimeFreqL * 1.4142, 0, chimeFreqL * 2);
    var chimeModR = SinOsc.ar(chimeFreqR * 1.4142, 0, chimeFreqR * 2);
    var chimeL = SinOsc.ar(chimeFreqL + chimeModL) * EnvGen.ar(Env.perc(0.001, 0.5), chimeTrigL) * 0.15;
    var chimeR = SinOsc.ar(chimeFreqR + chimeModR) * EnvGen.ar(Env.perc(0.001, 0.5), chimeTrigR) * 0.15;

    // Combine
    var sigL = pad + chirpL + chimeL;
    var sigR = pad + chirpR + chimeR;

    // Envelope
    var env = EnvGen.ar(Env.adsr(attack, decay, sustain, release), gateSig, doneAction: 14);

    Out.ar(out, [sigL, sigR] * finalAmp * velSig * env);
}).writeDefFile(dir);
)
