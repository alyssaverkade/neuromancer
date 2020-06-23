# Neuromancer

![Rust](https://github.com/alyssaverkade/neuromancer/workflows/Rust/badge.svg)

A mapreduce implementation that serializes the map and reduce transforms as LLVM bitcode (eventually).


### Code layout
* `/neuromancer`: the shared code that the other subprojects rely on
* `/executor`: the execution unit that runs the map and reduce jobs
* `/librarian`: the service for keeping track of what data lies where
* `/supervisor`: the control plane of the cluster

You can watch me work on this on my [stream](https://twitch.tv/alyssacodes) semioccasionaly!
