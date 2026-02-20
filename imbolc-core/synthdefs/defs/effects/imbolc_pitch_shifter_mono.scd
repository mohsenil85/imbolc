// imbolc_pitch_shifter_mono SynthDef
(
var dir = thisProcess.nowExecutingPath.dirname.dirname.dirname;

SynthDef(\imbolc_pitch_shifter_mono, { |in=1024, out=1026, shift=0, window=0.2, mix=0.5, lag=0.02,
    snap=0, ref_offset=0, q0=0, q1=0, q2=0, q3=0, q4=0, q5=0, q6=0, q7=0, q8=0, q9=0, q10=0, q11=0|
    var sig = In.ar(in, 1);
    var fadein = Line.kr(0, 1, 0.01);
    var freq, hasFreq, detectedMidi, targetMidi, pitchClass, scaleCorrection, quantizedTarget, ratio, wet;

    // Detect input pitch for scale-aware shifting
    #freq, hasFreq = Pitch.kr(sig, ampThreshold: 0.02, median: 7);
    detectedMidi = freq.cpsmidi + ref_offset.lag(lag);

    // Compute target = detected + shift
    targetMidi = detectedMidi + shift.lag(lag);

    // Quantize target to scale when snap=1
    pitchClass = targetMidi.round.mod(12);
    scaleCorrection = Select.kr(pitchClass, [q0, q1, q2, q3, q4, q5, q6, q7, q8, q9, q10, q11]);
    quantizedTarget = Select.kr(snap.round.clip(0, 1), [
        targetMidi,
        targetMidi.round + scaleCorrection.lag(lag)
    ]);

    // Compute ratio: when pitch detected use quantized ratio, otherwise raw shift
    ratio = Select.kr(hasFreq * snap.round.clip(0, 1), [
        2 ** (shift.lag(lag) / 12),
        (quantizedTarget - detectedMidi).midiratio
    ]);

    wet = PitchShift.ar(sig, window.lag(lag), ratio);
    Out.ar(out, ((sig * (1 - mix.lag(lag))) + (wet * mix.lag(lag))) * fadein);
}).writeDefFile(dir);
)
