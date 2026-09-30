# Python pack

The Python pack contains the `bridle_specs` pytest plugin (see `adapters/README.md`)
and linting and style rules for Python projects.

## Running bridle in CI

To run the specs in CI, add a step to download and install the bridle binary
(pinned to a specific release). This step must run before pytest collection.

Add this to your workflow (e.g., in `.github/workflows/test.yml`):

```yaml
- name: Install bridle
  env:
    BRIDLE_VERSION: v0.4.0
  run: |
    set -e
    case "${{ runner.os }}-${{ runner.arch }}" in
      Linux-X64)
        target=x86_64-unknown-linux-gnu
        ;;
      macOS-X64)
        target=x86_64-apple-darwin
        ;;
      macOS-ARM64)
        target=aarch64-apple-darwin
        ;;
      *)
        echo "unsupported platform: ${{ runner.os }}-${{ runner.arch }}"
        exit 1
        ;;
    esac
    
    cd /tmp
    curl -fsSL \
      https://github.com/jonathanbranam/bridle/releases/download/$BRIDLE_VERSION/bridle-$BRIDLE_VERSION-$target.tar.gz \
      -o bridle.tar.gz
    curl -fsSL \
      https://github.com/jonathanbranam/bridle/releases/download/$BRIDLE_VERSION/SHA256SUMS \
      -o SHA256SUMS
    
    grep " bridle-$BRIDLE_VERSION-$target.tar.gz" SHA256SUMS | shasum -a 256 -c -
    tar -xzf bridle.tar.gz
    mkdir -p ~/.local/bin
    mv bridle ~/.local/bin/
    echo "$HOME/.local/bin" >> $GITHUB_PATH
```

Adjust `BRIDLE_VERSION` to the release you want to use. The binary will be
added to `PATH` so pytest can find it during collection.
