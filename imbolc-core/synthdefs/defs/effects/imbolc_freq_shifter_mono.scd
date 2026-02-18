// imbolc_freq_shifter_mono SynthDef
(
var dir = thisProcess.nowExecutingPath.dirname.dirname.dirname;

SynthDef(\imbolc_freq_shifter_mono, { |in=1024, out=1026, shift_hz=0, snap=1, root_freq=261.62555, mix=0.5, lag=0.02, ref_offset=0,
    q0=0, q1=0, q2=0, q3=0, q4=0, q5=0, q6=0, q7=0, q8=0, q9=0, q10=0, q11=0|
    var sig = In.ar(in, 1);
    var fadein = Line.kr(0, 1, 0.01);
    var root, targetFreq, targetMidi, pitchClass, scaleCorrection, snappedShift, effectiveShift, wet;

    root = root_freq.lag(lag).max(20);
    targetFreq = (root + shift_hz.lag(lag)).max(20);
    targetMidi = targetFreq.cpsmidi + ref_offset.lag(lag);
    pitchClass = targetMidi.round.mod(12);
    scaleCorrection = Select.kr(pitchClass, [q0, q1, q2, q3, q4, q5, q6, q7, q8, q9, q10, q11]);
    snappedShift = ((targetMidi.round + scaleCorrection.lag(lag)).midicps - root);
    effectiveShift = Select.kr(snap.round.clip(0, 1), [shift_hz.lag(lag), snappedShift]);

    wet = FreqShift.ar(sig, effectiveShift);
    Out.ar(out, ((sig * (1 - mix.lag(lag))) + (wet * mix.lag(lag))) * fadein);
}).writeDefFile(dir);
)
