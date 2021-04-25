# ruffbox-synth
Separate repository for the rust-based beatbox synth/sampler.

This is not intended to have the flexibility of, say, SuperCollider. 

It's heavily guided by my own artistic practice, which follows an event-sequencing
paradigm, not a modular-synth, "everything-is-a-signal" paradigm.

That is, you can throw it events with certain parameters and it'll turn them into sound, 
but there's no inlets, outlets or anything that you could freely connect. If you want new sounds,
as of now you'd have to implement them yourself.

This also might lead to constraints that seem odd, like, the only available samplerate being 44100.
That's what I work in all the time, so I never put any priority on making more samplerates available.

The philosophy is to determine as much as possible at compile-time, using const generics.
