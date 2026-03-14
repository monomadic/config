{ lib, fetchFromGitHub, rustPlatform, rustc, cargo, clang, openssl, pkgconfig, rustup, glib, gstreamer, gstreamermm, gst_all_1 }:
# nix-build -E 'with import <nixpkgs> {}; callPackage ./cargo.nix {}'

rustPlatform.buildRustPackage rec {
  pname = "hunter";
  version = "v1.3.5";

  src = fetchFromGitHub {
    owner = "rabite0";
    repo = pname;
    rev = version;
    sha256 = "0z28ymz0kr726zjsrksipy7jz7y1kmqlxigyqkh3pyh154b38cis";
  };

  buildInputs = [
    rustup
    glib
    gstreamer
    gstreamermm
    gst_all_1.gstreamer
    gst_all_1.gst-plugins-base
    gst_all_1.gst-plugins-bad
    clang
    openssl
    pkgconfig
  ];

  doCheck = false;

  cargoSha256 = "1ylb7alnkfzb9xqdzkdc1q1lfnqzgqcp7bjmih5h1ylai8h34sdb";

  meta = with lib; {
    description = "Rust-based ranger replacement";
    homepage = "https://github.com/rabite0/hunter";
    license = licenses.unlicense;
    maintainers = [ maintainers.tailhook ];
    platforms = platforms.all;
  };
}