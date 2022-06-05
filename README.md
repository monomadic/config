# Config

## Installing

```
ln -s .dotter/nixos.toml.example .dotter/local.toml
dotter deploy
```

## NixOS

```
sudo ln -s $PWD/nixos/configuration.nix /etc/nixos/configuration.nix
sudo nixos-rebuild switch
```
