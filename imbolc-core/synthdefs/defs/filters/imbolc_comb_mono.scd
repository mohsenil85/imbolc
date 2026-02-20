// imbolc_comb_mono SynthDef
(
var dir = thisProcess.nowExecutingPath.dirname.dirname.dirname;

SynthDef(\imbolc_comb_mono, { |in=1024, out=1026, cutoff_mod_in=(-1), res_mod_in=(-1), cutoff=440, resonance=0.5, lag=0.02,
    snap=0, ref_offset=0, q0=0, q1=0, q2=0, q3=0, q4=0, q5=0, q6=0, q7=0, q8=0, q9=0, q10=0, q11=0|
    var sig = In.ar(in, 1);
    var fadein = Line.kr(0, 1, 0.01);
    var cutoffMod = Select.kr(cutoff_mod_in >= 0, [0, In.kr(cutoff_mod_in)]);
    var resMod = Select.kr(res_mod_in >= 0, [0, In.kr(res_mod_in)]);
    var finalCutoff = (cutoff.lag(lag) * (1 + cutoffMod)).clip(20, 20000);
    var finalRes = (resonance.lag(lag) + resMod).clip(0, 1);
    var baseMidi, pitchClass, scaleCorrection, targetMidi, quantizedFreq, delay, decayTime;

    // Quantize cutoff to scale when snap=1 (same pattern as imbolc_resonator)
    baseMidi = finalCutoff.cpsmidi + ref_offset.lag(lag);
    pitchClass = baseMidi.round.mod(12);
    scaleCorrection = Select.kr(pitchClass, [q0, q1, q2, q3, q4, q5, q6, q7, q8, q9, q10, q11]);
    targetMidi = Select.kr(snap.round.clip(0, 1), [
        baseMidi,
        baseMidi.round + scaleCorrection.lag(lag)
    ]);
    quantizedFreq = targetMidi.midicps.clip(20, 20000);

    delay = quantizedFreq.reciprocal.clip(0.00005, 0.05);
    decayTime = finalRes.linlin(0, 1, 0.01, 2.0);
    Out.ar(out, CombC.ar(sig, 0.05, delay, decayTime) * fadein);
}).writeDefFile(dir);
)
