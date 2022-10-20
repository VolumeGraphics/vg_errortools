[![Documentation](https://docs.rs/vg_errortools/badge.svg)](https://docs.rs/vg_errortools)
# vg_errortools - Small error utilities

This crate comprises mainly helpers for generating fat io errors - errors which carry the path of the file it failed with them.
It solves the problem with not knowing which file was being processed in async/await powered logged utilities. 
Be aware, that the fat errors are cloning a `PathBuf` to store the affected file.

For more comfort in main functions a `MainError` is provided with a blanket implementation to allow using all errors with the `?` Operator.

It's no rocket-science, but convenient.