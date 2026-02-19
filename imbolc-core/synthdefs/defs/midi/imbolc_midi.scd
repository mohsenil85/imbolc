// imbolc_midi SynthDef
(
var dir = thisProcess.nowExecutingPath.dirname.dirname.dirname;

SynthDef(\imbolc_midi, { |freq_out=0, gate_out=0, vel_out=0, note=60, freq=440, vel=0.8, gate=1, glide=0|
    var env = EnvGen.kr(Env([1, 1, 0], [0, 0.01], releaseNode: 1), gate, doneAction: 2);
    var freqOut = Select.kr(glide > 0, [freq, VarLag.kr(freq, glide)]);
    ReplaceOut.kr(freq_out, freqOut);
    ReplaceOut.kr(gate_out, env);
    ReplaceOut.kr(vel_out, vel);
}).writeDefFile(dir);
)
