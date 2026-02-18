// imbolc_ringmod SynthDef
(
var dir = thisProcess.nowExecutingPath.dirname.dirname.dirname;

SynthDef(\imbolc_ringmod, { |in=1024, out=1026, freq=440, snap=1, depth=0.5, mix=0.5, lag=0.02, ref_offset=0,
    q0=0, q1=0, q2=0, q3=0, q4=0, q5=0, q6=0, q7=0, q8=0, q9=0, q10=0, q11=0|
    var sig = In.ar(in, 2);
    var fadein = Line.kr(0, 1, 0.01);
    var baseMidi, pitchClass, scaleCorrection, targetMidi, carrierFreq, mod;

    baseMidi = freq.lag(lag).clip(20, 5000).cpsmidi + ref_offset.lag(lag);
    pitchClass = baseMidi.round.mod(12);
    scaleCorrection = Select.kr(pitchClass, [q0, q1, q2, q3, q4, q5, q6, q7, q8, q9, q10, q11]);
    targetMidi = Select.kr(snap.round.clip(0, 1), [
        baseMidi,
        baseMidi.round + scaleCorrection.lag(lag)
    ]);
    carrierFreq = targetMidi.midicps.clip(20, 5000);
    mod = SinOsc.ar(carrierFreq);

    var wet = sig * ((1 - depth.lag(lag)) + (mod * depth.lag(lag)));
    Out.ar(out, ((sig * (1 - mix.lag(lag))) + (wet * mix.lag(lag))) * fadein);
}).writeDefFile(dir);
)
