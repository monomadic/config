{ lib, fetchgit, buildGoModule }:
# with (import <nixpkgs> { });

buildGoModule rec {
  name = "mailygo";

  src = fetchgit {
    url = "https://codeberg.org/jlelse/MailyGo.git";
    rev = "78eaed64d5b2fb33cd626bf364f561bd729aa371";
    sha256 = "0bp66yv51wm9jl1qwy71vd5xyqrhhws66s75rcy4irbnhc2lsi5p";
  };

  modSha256 = "0r152h87isnhs9psg2k924vgn3bv2mfb6zsmd47ncg5am0jx12z8";

  meta = with lib; {
    description =
      "MailyGo allows to send HTML forms, for example from static websites without a dynamic backend, via email";
    homepage = "https://codeberg.org/jlelse/MailyGo";
    license = licenses.mit;
    #   maintainers = with maintainers; [ your-name-here ];
    platforms = platforms.linux ++ platforms.darwin;
  };
}
