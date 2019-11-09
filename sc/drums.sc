(
    s = Server.local.boot;
    MIDIClient.init(1, 0, verbose: true);
)

(
    SynthDef(\sampler, { |t_trig, bufnum, out = 0, freq = 1, amp = 1|
        var sig = PlayBuf.ar(1, bufnum, rate: freq, trigger: t_trig) * Lag.kr(amp);
        Out.ar(out, sig ! 2)
    }).add;
)

(
    ~base = "/usr/local/lib/lv2/fabla808.lv2";
    ~buffers = List[
        "Classic-808_Kick_long.wav",
        "Classic-808_Kick_short.wav",
        "Classic-808_Snare_lo1.wav",
        "Classic-808_Snare_lo2.wav",

        "Classic-808_Snare_lo3.wav",
        "Classic-808_Rim_Shot.wav",
        "Classic-808_Cowbell.wav",
        "Classic-808_Clap.wav",

        "Classic-808_Hat_closed.wav",
        "Classic-808_Hat_long.wav",
        "Classic-808_Clave.wav",
        "Classic-808_Cymbal-high.wav",

        "Classic-808_Hi_Tom.wav",
        "Classic-808_Md_Tom.wav",
        "Classic-808_Lo_Tom.wav",
        "Classic-808_Maracas.wav",

        "Classic-808_Hi_Conga.wav",
        "Classic-808_Md_Conga.wav",
        "Classic-808_Lo_Conga.wav",
    ].collect({
        arg item, i;
        Buffer.read(s, ~base +/+ item);
    });
)
(
    ~synths = ~buffers.collect({
        arg item, i;
        Synth(\sampler, [t_trig: 0, bufnum: item.bufnum, amp: 0])
    }); 
)
~synths[0].set(\t_trig, 1, \amp, 0.9);
(
    ~noteon = MIDIFunc.noteOn({
        arg val, num, chan, src;
        if (num < ~synths.size, {
            ~synths[num].set(\t_trig, 1, \amp, val / 127.0);
        });
    }, chan: 0);
)
~noteon.free;

s.freeAll;

