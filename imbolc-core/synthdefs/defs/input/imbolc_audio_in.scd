// imbolc_audio_in SynthDef — with cascaded feedback suppression
(
var dir = thisProcess.nowExecutingPath.dirname.dirname.dirname;

SynthDef(\imbolc_audio_in, { |out=1024, channel=0, gain=1, test_tone=0, test_freq=440,
    instrument_id=0, fb_suppress=0, fb_threshold=0.5, fb_q=0.005|

    var input = SoundIn.ar(channel) * gain;
    var test = SinOsc.ar(test_freq) * 0.3 * test_tone;
    var raw = input + test;

    // --- Cascaded feedback detection & suppression (8 slots) ---
    var analysis = raw;
    var processed = raw;
    var freq, hasFreq, ampFast, ampSlow, freqFast, freqSlow, stable, active;

    8.do {
        // Detect dominant periodic frequency in the (residual) analysis signal
        #freq, hasFreq = Pitch.kr(analysis, initFreq: 440, minFreq: 60, maxFreq: 8000,
            ampThreshold: 0.01, median: 3);

        // Amplitude at detected frequency: fast tracker (attack) and slow tracker (baseline)
        ampFast = Amplitude.kr(BPF.ar(analysis, freq.clip(60, 8000), 0.05), attackTime: 0.1, releaseTime: 0.1);
        ampSlow = Amplitude.kr(BPF.ar(analysis, freq.clip(60, 8000), 0.05), attackTime: 2.0, releaseTime: 2.0);

        // Frequency stability: feedback locks on; music drifts
        freqFast = Lag.kr(freq, 0.3);
        freqSlow = Lag.kr(freq, 1.5);
        stable = (freqFast - freqSlow).abs < (freqSlow * 0.02); // <2% drift = stable

        // Activate when: pitch detected AND amplitude above threshold
        //                AND amplitude sustained/growing AND frequency stable
        active = (hasFreq > 0.5) * (ampFast > fb_threshold) * (ampFast >= (ampSlow * 0.9)) * stable;
        active = Lag.kr(active, 0.05); // smooth activation to avoid clicks

        // Remove detected frequency from analysis signal for next cascade stage
        analysis = BRF.ar(analysis, freq.clip(60, 8000), 0.01);

        // Apply notch to output: rq interpolates between 1.0 (transparent) and fb_q (narrow notch)
        processed = BRF.ar(processed, freq.clip(60, 8000), active.linlin(0, 1, 1.0, fb_q));
    };

    // Bypass: crossfade between raw and processed based on fb_suppress
    processed = (raw * (1 - fb_suppress)) + (processed * fb_suppress);

    // Send peak amplitude for waveform display (10 times per second)
    SendPeakRMS.kr(processed, 10, 3, "/audio_in_level", instrument_id);
    Out.ar(out, processed ! 2);
}).writeDefFile(dir);
)
