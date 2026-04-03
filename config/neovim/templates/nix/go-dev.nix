{ pkgs ? import <nixpkgs> {} }:

with pkgs;
buildGoModule rec {
  pname = "dstask";
  version = "0.0.1";
  src = nix-gitignore.gitignoreSource [] ./.;
  goPackagePath = "github.com/monomadic/${pname}";
  vendorSha256 = null;

  shellHook = ''
    set -v
    export GOPATH="$(pwd)/.go"
    export GOCACHE=""
    export GO111MODULE='on'
    go mod init ${goPackagePath}
    set +v
  '';

  buildPhase = ''
    go build cmd/dstask.go
  '';

  installPhase = ''
    mkdir -p $out/bin
    find .
    cp ./${pname} $out/bin/${pname}
    chmod +x $out/bin/${pname}
  '';
}
