## Building

```
git clone ...
git submodule update --init --recursive
```

The build.rs file does modify the submodule in order to use only path
dependencies. This is done to avoid running into compilation issues with
duplicated git dependencies.
