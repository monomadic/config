class Dotter < Formula
  desc "A dotfile manager and templater written in rust"
  homepage "https://github.com/SuperCuber/dotter"
  url "https://github.com/SuperCuber/dotter/archive/refs/tags/v0.12.15.tar.gz"
  sha256 "fbe0236a555f88646bd1bdcbf3a1e09ba9de557987ead7cbac33d0b946c84ffc"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "build", "--release"
    bin.install "target/release/dotter"
  end

  test do
    assert_match "Expected output", shell_output("#{bin}/dotter --version")
  end
end
