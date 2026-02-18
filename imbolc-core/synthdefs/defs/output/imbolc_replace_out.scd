// imbolc_replace_out SynthDef
// Simple stereo copy with ReplaceOut — reads from `in` bus, replaces `out` bus.
// Used as the final stage of the master effect chain to write processed
// audio back to bus 0 without doubling.
(
var dir = thisProcess.nowExecutingPath.dirname.dirname.dirname;

SynthDef(\imbolc_replace_out, { |in=1024, out=0|
    ReplaceOut.ar(out, In.ar(in, 2));
}).writeDefFile(dir);
)
