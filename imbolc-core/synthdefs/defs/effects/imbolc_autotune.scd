// imbolc_autotune SynthDef
(
var dir = thisProcess.nowExecutingPath.dirname.dirname.dirname;

SynthDef(\imbolc_autotune, { |in=1024, out=1026, speed=0.5, shift=0, snap=1, mix=1.0, lag=0.02, ref_offset=0,
    q0=0, q1=0, q2=0, q3=0, q4=0, q5=0, q6=0, q7=0, q8=0, q9=0, q10=0, q11=0|
    var sig = In.ar(in, 2);
    var fadein = Line.kr(0, 1, 0.01);
    var mono = (sig[0] + sig[1]) * 0.5;
    var freq, hasFreq, midi, pitchClass, scaleCorrection, targetMidi, correction, ratio, corrected, wet;

    // Detect pitch
    #freq, hasFreq = Pitch.kr(mono, ampThreshold: 0.02, median: 7);

    // Convert to MIDI and correct for non-440 tuning reference
    midi = freq.cpsmidi + ref_offset.lag(lag);

    // Quantize: chromatic (snap=0) or scale-aware via per-pitch-class table (snap=1)
    pitchClass = midi.round.mod(12);
    scaleCorrection = Select.kr(pitchClass, [q0, q1, q2, q3, q4, q5, q6, q7, q8, q9, q10, q11]);
    targetMidi = Select.kr(snap.round.clip(0, 1), [
        midi.round,
        midi.round + scaleCorrection.lag(lag)
    ]) + shift.lag(lag);

    // Interpolate between detected and target based on speed
    correction = (targetMidi - midi) * speed.lag(lag).linlin(0, 1, 0.1, 1.0);
    ratio = correction.midiratio;

    // Apply pitch shift (only when pitch detected)
    corrected = PitchShift.ar(mono, 0.2, ratio * hasFreq + (1 - hasFreq), 0, 0.004);
    wet = corrected ! 2;

    Out.ar(out, ((sig * (1 - mix.lag(lag))) + (wet * mix.lag(lag))) * fadein);
}).writeDefFile(dir);
)
