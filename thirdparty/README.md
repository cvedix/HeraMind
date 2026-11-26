# Third-party dependencies

`engine/` is a Git submodule from https://github.com/cvedix/engine.git,
configured to track `main`. HeraMind records an exact engine commit so
checkouts use a reproducible version.

Initialize the dependency after checking out this branch:

```sh
git submodule update --init --recursive thirdparty/engine
```

To update to the latest commit on the engine's `main` branch:

```sh
git submodule update --remote thirdparty/engine
git add thirdparty/engine
git commit -m "chore: update engine submodule"
```
