#  mahou_vmc

This is an implementation of the [Virtual Motion Capture Protocol](https://protocol.vmc.info/).

The crate is "sans-io", which means it does not perform any I/O by itself, and instead just
provides a way to parse incoming data (in the form of OSC packets), which can be interpreted
and applied to internal data structures.

The actual API of this crate is still in flux and is primarily intended to support other
Mahou Technologies applications. That being said, community uses of this crate are encouraged,
and third-party use cases will be considered and supported.

