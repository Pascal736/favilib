with import <nixpkgs> { };
stdenv.mkDerivation {
  name = "yabm-dev-environment";
  buildInputs = [ pkg-config openssl ];
}
