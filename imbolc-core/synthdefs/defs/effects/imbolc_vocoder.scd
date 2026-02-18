// imbolc_vocoder SynthDef
(
var dir = thisProcess.nowExecutingPath.dirname.dirname.dirname;

SynthDef(\imbolc_vocoder, { |in=1024, out=1026, bands=16, snap=1, root_freq=261.62555, mix=0.5, lag=0.02|
    var sig = In.ar(in, 2);
    var fadein = Line.kr(0, 1, 0.01);
    var mono = (sig[0] + sig[1]) * 0.5;
    var wet = Mix.fill(16, { |i|
        var active = (i < bands);
        var base = Select.kr(snap.round.clip(0, 1), [100, root_freq.lag(lag).max(20)]);
        var freq = (base * (2 ** (i * 0.5))).min(16000);
        var analysis = Amplitude.kr(BPF.ar(mono, freq, 0.1), 0.01, 0.05);
        SinOsc.ar(freq) * analysis * active;
    });
    wet = [wet, wet];
    Out.ar(out, ((sig * (1 - mix.lag(lag))) + (wet * mix.lag(lag))) * fadein);
}).writeDefFile(dir);
)
