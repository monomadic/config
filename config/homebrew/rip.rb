class Rip < Formula
  desc "A safe and ergonomic rust alternative to rm"
  homepage "https://github.com/nivekuil/rip"
  url "https://github.com/nivekuil/rip/archive/refs/tags/0.13.1.tar.gz"
  sha256 "73acdc72386242dced117afae43429b6870aa176e8cc81e11350e0aaa95e6421"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "build", "--release"
    bin.install "target/release/rip"
  end

  test do
    assert_match "Expected output", shell_output("#{bin}/rip --version")
  end
end
