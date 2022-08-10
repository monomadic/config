# Config

## Installing

```
# install dotter
wget https://github.com/SuperCuber/dotter/releases/download/v0.12.13/dotter

# or install rustup and compile it
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install dotter

# install config
ln -s .dotter/nixos.toml.example .dotter/local.toml
dotter deploy
```

## NixOS

```
sudo ln -s $PWD/nixos/configuration.nix /etc/nixos/configuration.nix
sudo nixos-rebuild switch
```
