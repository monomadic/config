# working shell

{ pkgs ? import <nixpkgs> {} }:

with pkgs;
buildGoModule rec {
  pname = "golang-dev";
  version = "0.0.1";
  src = nix-gitignore.gitignoreSource [] ./.;
  goPackagePath = "github.com/monomadic/${pname}";
  vendorSha256 = null;
  shellHook = ''
    echo 'Entering ${pname}'
    set -v
    export GOPATH="$(pwd)/.go"
    export GOCACHE=""
    export GO111MODULE='on'
    go mod init ${goPackagePath}
    set +v
  '';
}


#


with import <nixpkgs> {}; with goPackages;

buildGoPackage rec {
  name = "yourproject";
  buildInputs = [ net osext ];
  goPackagePath = "github.com/you/yourproject";
}

# packages:
{ goPackages, fetchFromGitHub }:

goPackages.buildGoPackage rec {
  rev = "67e2db24c831afa6c64fc17b4a143390674365ef";
  name = "pty-${rev}";
  goPackagePath = "github.com/kr/pty";
  src = fetchFromGitHub {
    inherit rev;
    owner = "kr";
    repo = "pty";
    sha256 = "1l3z3wbb112ar9br44m8g838z0pq2gfxcp5s3ka0xvm1hjvanw2d";
  };
}`