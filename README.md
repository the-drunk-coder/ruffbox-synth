# ruffbox-synth
Separate repository for the rust-based beatbox synth/sampler.

This is not intended to have the flexibility of, say, SuperCollider.
It's heavily guided by my own artistic practice, which follows an event-sequencing
paradigm, not a modular-synth, "everything-is-a-signal" paradigm.
This also might lead to constraints that seem odd, like, the only available samplerate being 44100.
That's what I work in all the time, so I never put any priority on making more samplerates available.

Currently it depends heavily on experimental features (const generics). 

The philosophy is "determine as much as possible at compile-time".
