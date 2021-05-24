# Mimic is a 3D rendering engine

Work is in progress. 

At the moment the repostitory contains the basic setup of a Vulkan rendering backend

TODO: A rendering frontend based on the render graph concept

## How to build and run
### Building
resources are copied and shaders are compiled as part of the build.rs process
to see the output of this either go to the crate directory
target/build/$crate-$hash/output
or just pass `-vv` to cargo

Passing `-vv` results in a lot of dependency and build command noise so it may be better to only do that when running cargo build not when doing run
### Running
to run the example in mimic_frontend (not the bin in mimic_vulkan_backend) 

With logging and backtrace 
```
$env:RUST_BACKTRACE = 1; $env:RUST_LOG = "info"; cargo run --example demo
```


