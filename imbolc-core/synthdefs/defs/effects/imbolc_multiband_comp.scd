// imbolc_multiband_comp SynthDef
(
var dir = thisProcess.nowExecutingPath.dirname.dirname.dirname;

SynthDef(\imbolc_multiband_comp, { |in=1024, out=1026, low_thresh=0.3, mid_thresh=0.3, hi_thresh=0.3, snap=1, root_freq=261.62555, mix=1.0, lag=0.02|
    var sig = In.ar(in, 2);
    var fadein = Line.kr(0, 1, 0.01);
    var root, lowCut, midCenter, hiCut, lo, mid, hi;

    root = root_freq.lag(lag).max(20);
    lowCut = Select.kr(snap.round.clip(0, 1), [250, root.clip(120, 500)]);
    midCenter = Select.kr(snap.round.clip(0, 1), [2000, (root * 8).clip(800, 4000)]);
    hiCut = Select.kr(snap.round.clip(0, 1), [6000, (root * 24).clip(2500, 12000)]);

    lo = LPF.ar(sig, lowCut);
    mid = BPF.ar(sig, midCenter, 1);
    hi = HPF.ar(sig, hiCut);
    var wet = Compander.ar(lo, lo, low_thresh.lag(lag), 1, 0.25, 0.01, 0.1)
            + Compander.ar(mid, mid, mid_thresh.lag(lag), 1, 0.25, 0.01, 0.1)
            + Compander.ar(hi, hi, hi_thresh.lag(lag), 1, 0.25, 0.01, 0.1);
    Out.ar(out, ((sig * (1 - mix.lag(lag))) + (wet * mix.lag(lag))) * fadein);
}).writeDefFile(dir);
)
