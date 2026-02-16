// imbolc_soundtrack SynthDef — Roland JV-1080 "Soundtrack" Board style
// Cinematic strings + brass pad + sub
(
var dir = thisProcess.nowExecutingPath.dirname.dirname.dirname;

SynthDef(\imbolc_soundtrack, { |out=1024, freq_in=(-1), gate_in=(-1), vel_in=(-1),
    freq=440, amp=0.5, lag=0.02, brass=0.4, sweep=0.5, width=0.5,
    attack=0.6, decay=0.2, sustain=0.8, release=0.8,
    amp_mod_in=(-1), pitch_mod_in=(-1)|

    var freqSig = Select.kr(freq_in >= 0, [freq.lag(lag), In.kr(freq_in)]);
    var gateSig = Select.kr(gate_in >= 0, [1, In.kr(gate_in)]);
    var velSig = Select.kr(vel_in >= 0, [1, In.kr(vel_in)]);
    var ampMod = Select.kr(amp_mod_in >= 0, [0, In.kr(amp_mod_in)]);
    var pitchMod = Select.kr(pitch_mod_in >= 0, [0, In.kr(pitch_mod_in)]);
    var finalFreq = freqSig * (2 ** pitchMod);
    var finalAmp = amp.lag(lag) * (1 + ampMod).max(0);
    var finalBrass = brass.lag(lag);
    var finalSweep = sweep.lag(lag);
    var finalWidth = width.lag(lag);

    // Layer 1: Strings — 6 detuned saws through LPF with slow sweep
    var stringSaws = Array.fill(6, { |i|
        var dt = 0.01 * (i - 2.5) / 2.5;
        Saw.ar(finalFreq * (1 + dt));
    });
    var stringMix = stringSaws.sum / 6;
    var stringLfo = SinOsc.kr(0.12);
    var stringFilterFreq = LinLin.kr(stringLfo * finalSweep, -1, 1, 800, 4000);
    var strings = LPF.ar(stringMix, stringFilterFreq.clip(100, 12000));

    // Layer 2: Brass — pulse + saw blend through resonant LPF (phase-offset from strings)
    var brassPulse = Pulse.ar(finalFreq, 0.45) * 0.5;
    var brassSaw = Saw.ar(finalFreq * 1.002) * 0.5;
    var brassRaw = brassPulse + brassSaw;
    var brassLfo = SinOsc.kr(0.15, pi * 0.7); // phase offset from string LFO
    var brassFilterFreq = LinLin.kr(brassLfo * finalSweep, -1, 1, 600, 3500);
    var brassSig = RLPF.ar(brassRaw, brassFilterFreq.clip(100, 12000), 0.6) * finalBrass;

    // Layer 3: Sub — sine at half frequency for weight
    var sub = SinOsc.ar(finalFreq * 0.5) * 0.25;

    // Combine
    var sig = strings + brassSig + sub;

    // Envelope
    var env = EnvGen.ar(Env.adsr(attack, decay, sustain, release), gateSig, doneAction: 14);

    // Stereo width via delay-based spreading
    var delayTime = LinLin.kr(finalWidth, 0, 1, 0.0, 0.012);
    var sigL = sig;
    var sigR = DelayC.ar(sig, 0.02, delayTime);

    Out.ar(out, [sigL, sigR] * finalAmp * velSig * env);
}).writeDefFile(dir);
)
