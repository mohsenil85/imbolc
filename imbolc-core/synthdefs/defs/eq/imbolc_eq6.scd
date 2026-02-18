// imbolc_eq6 SynthDef
// 6-band parametric EQ: HPF, LowShelf, Bell, Bell, HighShelf, LPF
// Band types: 0=HP, 1=LS, 2=Bell, 3=HS, 4=LP
// Slope (HPF/LPF only): 1=12dB/oct, 2=24dB/oct, 3=48dB/oct
(
var dir = thisProcess.nowExecutingPath.dirname.dirname.dirname;

SynthDef(\imbolc_eq6, { |in=1024, out=1026, lag=0.02,
    b0_freq=20,    b0_gain=0, b0_q=0.7, b0_on=1, b0_type=0, b0_slope=1,
    b1_freq=100,   b1_gain=0, b1_q=0.7, b1_on=1, b1_type=1, b1_slope=1,
    b2_freq=500,   b2_gain=0, b2_q=1.0, b2_on=1, b2_type=2, b2_slope=1,
    b3_freq=2500,  b3_gain=0, b3_q=1.0, b3_on=1, b3_type=2, b3_slope=1,
    b4_freq=8000,  b4_gain=0, b4_q=0.7, b4_on=1, b4_type=3, b4_slope=1,
    b5_freq=20000, b5_gain=0, b5_q=0.7, b5_on=1, b5_type=4, b5_slope=1|

    var sig = In.ar(in, 2);
    var fadein = Line.kr(0, 1, 0.01);

    // Helper function: process one parametric band
    // type: 0=HP, 1=LS, 2=Bell, 3=HS, 4=LP
    // slope: 1=12dB, 2=24dB, 3=48dB (only for HP/LP)
    var processBand = { |input, freq, gain, q, on, type, slope|
        var f, g, rq, processed;
        var hp1, hp2, hp4, hpOut;
        var lp1, lp2, lp4, lpOut;
        var ls, bell, hs;

        f = freq.lag(lag).clip(20, 20000);
        g = gain.lag(lag);
        rq = q.lag(lag).reciprocal.clip(0.1, 10);

        // High-pass with slope cascading
        hp1 = BHiPass.ar(input, f, rq);
        hp2 = BHiPass.ar(hp1, f, rq);
        hp4 = BHiPass.ar(BHiPass.ar(hp2, f, rq), f, rq);
        hpOut = Select.ar(slope - 1, [hp1, hp2, hp4]);

        // Low shelf
        ls = BLowShelf.ar(input, f, rq, g);

        // Bell (peaking)
        bell = BPeakEQ.ar(input, f, rq, g);

        // High shelf
        hs = BHiShelf.ar(input, f, rq, g);

        // Low-pass with slope cascading
        lp1 = BLowPass.ar(input, f, rq);
        lp2 = BLowPass.ar(lp1, f, rq);
        lp4 = BLowPass.ar(BLowPass.ar(lp2, f, rq), f, rq);
        lpOut = Select.ar(slope - 1, [lp1, lp2, lp4]);

        // Select by type: 0=HP, 1=LS, 2=Bell, 3=HS, 4=LP
        processed = Select.ar(type, [hpOut, ls, bell, hs, lpOut]);

        // Bypass: on=0 -> dry, on=1 -> processed
        Select.ar(on, [input, processed]);
    };

    // Band 0 (default: HPF)
    sig = processBand.(sig, b0_freq, b0_gain, b0_q, b0_on, b0_type, b0_slope);
    // Band 1 (default: Low Shelf)
    sig = processBand.(sig, b1_freq, b1_gain, b1_q, b1_on, b1_type, b1_slope);
    // Band 2 (default: Bell)
    sig = processBand.(sig, b2_freq, b2_gain, b2_q, b2_on, b2_type, b2_slope);
    // Band 3 (default: Bell)
    sig = processBand.(sig, b3_freq, b3_gain, b3_q, b3_on, b3_type, b3_slope);
    // Band 4 (default: High Shelf)
    sig = processBand.(sig, b4_freq, b4_gain, b4_q, b4_on, b4_type, b4_slope);
    // Band 5 (default: LPF)
    sig = processBand.(sig, b5_freq, b5_gain, b5_q, b5_on, b5_type, b5_slope);

    Out.ar(out, sig * fadein);
}).writeDefFile(dir);
)
