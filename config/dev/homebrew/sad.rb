class Sad < Formula
  desc "CLI search and replace"
  homepage "https://github.com/ms-jpq/sad"
  url "https://github.com/ms-jpq/sad/releases/download/v0.4.22/aarch64-apple-darwin.zip"
  sha256 "2f217c019a666b32ca50df3f0412c375714dd31790764857402254ae77b07966"

  def install
    bin.install "sad"
  end

  test do
    system "#{bin}/sad", "--version"
  end
end
