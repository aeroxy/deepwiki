class Deepwiki < Formula
  desc "Query GitHub repository wikis via DeepWiki from the terminal"
  homepage "https://github.com/aeroxy/deepwiki"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/aeroxy/deepwiki/releases/download/#{version}/deepwiki_macos_arm64.zip"
      sha256 "dda8477b81064e97dc7231921ceb099dd8118bb6088e74bf59b5734742b814c2"
    end
  end

  def install
    bin.install "deepwiki"
  end

  test do
    assert_match "deepwiki #{version}", shell_output("#{bin}/deepwiki --version")
  end
end