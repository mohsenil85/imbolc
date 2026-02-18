// imbolc_wah_pedal SynthDef
(
var dir = thisProcess.nowExecutingPath.dirname.dirname.dirname;

SynthDef(\imbolc_wah_pedal, { |in=1024, out=1026, position=0.5, resonance=3, mode=0, snap=1, sensitivity=0.5, mix=1.0, lag=0.02, ref_offset=0,
    q0=0, q1=0, q2=0, q3=0, q4=0, q5=0, q6=0, q7=0, q8=0, q9=0, q10=0, q11=0|
    var sig = In.ar(in, 2);
    var fadein = Line.kr(0, 1, 0.01);
    var mono = (sig[0] + sig[1]) * 0.5;
    var env, freqCtrl, freq, freqMidi, pitchClass, scaleCorrection, rq, wet;

    // Envelope follower for auto-wah
    env = Amplitude.kr(mono, 0.01, 0.1) * sensitivity.lag(lag) * 4;

    // Choose control source: manual position or envelope
    freqCtrl = Select.kr(mode, [position.lag(lag), env.clip(0, 1)]);

    // Map to classic wah frequency range (300Hz - 2500Hz)
    freq = freqCtrl.linexp(0, 1, 300, 2500);

    // Optional scale snapping for the swept frequency
    freqMidi = freq.cpsmidi + ref_offset.lag(lag);
    pitchClass = freqMidi.round.mod(12);
    scaleCorrection = Select.kr(pitchClass, [q0, q1, q2, q3, q4, q5, q6, q7, q8, q9, q10, q11]);
    freq = Select.kr(snap.round.clip(0, 1), [
        freq,
        (freqMidi.round + scaleCorrection.lag(lag)).midicps.clip(300, 2500)
    ]);

    // Convert resonance to reciprocal Q (rq)
    rq = resonance.lag(lag).reciprocal;

    // Resonant bandpass filter
    wet = BPF.ar(sig, freq, rq) * resonance.lag(lag).sqrt;  // Compensate gain

    Out.ar(out, ((sig * (1 - mix.lag(lag))) + (wet * mix.lag(lag))) * fadein);
}).writeDefFile(dir);
)
