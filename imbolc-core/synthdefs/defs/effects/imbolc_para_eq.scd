// imbolc_para_eq SynthDef
(
var dir = thisProcess.nowExecutingPath.dirname.dirname.dirname;

SynthDef(\imbolc_para_eq, { |in=1024, out=1026,
    lo_freq=200, lo_gain=0, mid_freq=1000, mid_gain=0, hi_freq=5000, hi_gain=0, snap=1, lag=0.02, ref_offset=0,
    q0=0, q1=0, q2=0, q3=0, q4=0, q5=0, q6=0, q7=0, q8=0, q9=0, q10=0, q11=0|
    var sig = In.ar(in, 2);
    var fadein = Line.kr(0, 1, 0.01);
    var quantizeFreq, loFreq, midFreq, hiFreq;

    quantizeFreq = { |f|
        var midi = f.cpsmidi + ref_offset.lag(lag);
        var pitchClass = midi.round.mod(12);
        var scaleCorrection = Select.kr(pitchClass, [q0, q1, q2, q3, q4, q5, q6, q7, q8, q9, q10, q11]);
        Select.kr(snap.round.clip(0, 1), [f, (midi.round + scaleCorrection.lag(lag)).midicps]);
    };

    loFreq = quantizeFreq.(lo_freq.lag(lag).clip(20, 20000)).clip(20, 20000);
    midFreq = quantizeFreq.(mid_freq.lag(lag).clip(20, 20000)).clip(20, 20000);
    hiFreq = quantizeFreq.(hi_freq.lag(lag).clip(20, 20000)).clip(20, 20000);

    sig = BPeakEQ.ar(sig, loFreq, 1, lo_gain.lag(lag));
    sig = BPeakEQ.ar(sig, midFreq, 1, mid_gain.lag(lag));
    sig = BPeakEQ.ar(sig, hiFreq, 1, hi_gain.lag(lag));
    Out.ar(out, sig * fadein);
}).writeDefFile(dir);
)
