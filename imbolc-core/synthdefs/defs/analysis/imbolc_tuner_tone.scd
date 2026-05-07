// imbolc_tuner_tone SynthDef
// Reference tone for instrument tuning.
// Outputs directly to hardware out (bus 0), bypassing instrument routing.
// gate=1 to play, gate=0 to release (50ms fade, self-frees via doneAction:2).
// bright=0 pure sine, bright=1 adds harmonics 2/3/4 for low-note audibility.
(
var dir = thisProcess.nowExecutingPath.dirname.dirname.dirname;

SynthDef(\imbolc_tuner_tone, { |out=0, freq=440, amp=0.3, gate=1, bright=0|
    var sig, env, harmonics;
    env = EnvGen.kr(Env.asr(0.01, 1, 0.05), gate, doneAction: 2);
    sig = SinOsc.ar(freq);
    harmonics = SinOsc.ar(freq * 2, 0, 0.3)
              + SinOsc.ar(freq * 3, 0, 0.15)
              + SinOsc.ar(freq * 4, 0, 0.08);
    sig = sig + (harmonics * bright);
    sig = sig * amp * env;
    Out.ar(out, sig ! 2);
}).writeDefFile(dir);
)
