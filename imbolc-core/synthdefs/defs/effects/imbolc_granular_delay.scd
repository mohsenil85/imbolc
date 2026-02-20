// imbolc_granular_delay SynthDef
(
var dir = thisProcess.nowExecutingPath.dirname.dirname.dirname;

SynthDef(\imbolc_granular_delay, { |in=1024, out=1026, time=0.3, grain_size=0.1, density=10, pitch=0, mix=0.5, lag=0.02,
    snap=0, ref_offset=0, q0=0, q1=0, q2=0, q3=0, q4=0, q5=0, q6=0, q7=0, q8=0, q9=0, q10=0, q11=0|
    var sig = In.ar(in, 2);
    var fadein = Line.kr(0, 1, 0.01);
    var mono = (sig[0] + sig[1]) * 0.5;
    var bufFrames = 44100 * 2; // 2 second buffer
    var buf = LocalBuf(bufFrames, 1);
    var phase = Phasor.ar(0, 1, 0, bufFrames);
    var readPos, pitchRatio, trig, wet;
    var freq, hasFreq, detectedMidi, targetMidi, pitchClass, scaleCorrection, quantizedTarget;
    BufWr.ar(mono, buf, phase);
    readPos = (phase - (time.lag(lag) * SampleRate.ir)).wrap(0, bufFrames);

    // Detect input pitch for scale-aware grain pitch
    #freq, hasFreq = Pitch.kr(mono, ampThreshold: 0.02, median: 7);
    detectedMidi = freq.cpsmidi + ref_offset.lag(lag);

    // Compute target = detected + pitch param
    targetMidi = detectedMidi + pitch.lag(lag);

    // Quantize target to scale when snap=1
    pitchClass = targetMidi.round.mod(12);
    scaleCorrection = Select.kr(pitchClass, [q0, q1, q2, q3, q4, q5, q6, q7, q8, q9, q10, q11]);
    quantizedTarget = Select.kr(snap.round.clip(0, 1), [
        targetMidi,
        targetMidi.round + scaleCorrection.lag(lag)
    ]);

    // Compute ratio: when pitch detected use quantized ratio, otherwise raw pitch
    pitchRatio = Select.kr(hasFreq * snap.round.clip(0, 1), [
        2 ** (pitch.lag(lag) / 12),
        (quantizedTarget - detectedMidi).midiratio
    ]);

    trig = Impulse.ar(density);
    wet = TGrains.ar(2, trig, buf, pitchRatio, readPos / SampleRate.ir, grain_size.lag(lag), 0, 0.5);
    Out.ar(out, ((sig * (1 - mix.lag(lag))) + (wet * mix.lag(lag))) * fadein);
}).writeDefFile(dir);
)
