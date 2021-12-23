# ruffbox-synth
Separate repository for the rust-based beatbox synth/sampler.

This is not intended to have the flexibility of, say, SuperCollider. 

It's heavily guided by my own artistic practice, which follows an event-sequencing
paradigm, not a modular-synth, "everything-is-a-signal" paradigm.

That is, you can throw it events with certain parameters and it'll turn them into sound, 
but there's no inlets, outlets or anything that you could freely connect. If you want new sounds,
as of now you'd have to implement them yourself.

The philosophy is to determine as much as possible (such as blocksize and channel numbers)
at compile-time, using const generics.
