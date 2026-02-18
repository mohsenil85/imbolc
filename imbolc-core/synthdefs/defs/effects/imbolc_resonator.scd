// imbolc_resonator SynthDef
(
var dir = thisProcess.nowExecutingPath.dirname.dirname.dirname;

SynthDef(\imbolc_resonator, { |in=1024, out=1026, freq=440, snap=1, decay=1.0, mix=0.5, lag=0.02, ref_offset=0,
    q0=0, q1=0, q2=0, q3=0, q4=0, q5=0, q6=0, q7=0, q8=0, q9=0, q10=0, q11=0|
    var sig = In.ar(in, 2);
    var fadein = Line.kr(0, 1, 0.01);
    var baseMidi, pitchClass, scaleCorrection, targetMidi, resonantFreq, delay;

    baseMidi = freq.lag(lag).clip(20, 5000).cpsmidi + ref_offset.lag(lag);
    pitchClass = baseMidi.round.mod(12);
    scaleCorrection = Select.kr(pitchClass, [q0, q1, q2, q3, q4, q5, q6, q7, q8, q9, q10, q11]);
    targetMidi = Select.kr(snap.round.clip(0, 1), [
        baseMidi,
        baseMidi.round + scaleCorrection.lag(lag)
    ]);
    resonantFreq = targetMidi.midicps.clip(20, 5000);
    delay = resonantFreq.reciprocal.clip(0.00005, 0.05);

    var wet = CombC.ar(sig, 0.05, delay, decay.lag(lag));
    Out.ar(out, ((sig * (1 - mix.lag(lag))) + (wet * mix.lag(lag))) * fadein);
}).writeDefFile(dir);
)
