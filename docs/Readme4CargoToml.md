# Why these settings are critical for your mission:

# panic = "abort": 
This is mandatory. Default Rust behavior includes stack unwinding (exception handling) logic, which introduces hidden branches and personality routines into your machine code. Aborting directly saves MWh by removing this unnecessary logic.

# lto = "fat" + codegen-units = 1: 
This forces the compiler to view your entire codebase as a single unit during optimization. It prevents the compiler from generating cross-module jumps that can ruin cache-line predictions.

# Minimal Dependencies: 
By limiting yourself to libc, you ensure that the only code execution outside of your while(1) loop is the thin wrapper around the Linux kernel syscalls needed for mlockall and setaffinity.