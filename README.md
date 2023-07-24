# Dotfiles

## MacOS (Apple Silicon)

```bash
# install dependencies
bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew bundle

# link configuration
ln -s .dotter/nixos.toml.example .dotter/local.toml
dotter deploy

# python deps
pip3 install -r python.deps
```

## NixOS

```bash
sudo ln -s $PWD/nixos/configuration.nix /etc/nixos/configuration.nix
sudo nixos-rebuild switch
```
