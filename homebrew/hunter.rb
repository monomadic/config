class Hunter < Formula
  desc "Rust based ranger clone"
  homepage "https://github.com/rabite0/hunter"
  url "https://github.com/rabite0/hunter/releases/download/v1.1.3-holy/hunter-v1.1.3-holy-linux-arm-64.tar.gz"
  sha256 "58b3d162ce0ca9622519ac301a5795d969ef1a1f5df08429c43eab6d35669616"

  def install
    bin.install "hunter"
  end

  test do
    system "#{bin}/hunter", "--version"
  end
end
