class Deepwiki < Formula
  desc "Query GitHub repository wikis via DeepWiki from the terminal"
  homepage "https://github.com/aeroxy/deepwiki"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/aeroxy/deepwiki/releases/download/#{version}/deepwiki_macos_arm64.zip"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000" # Placeholder
    end
  end

  def install
    bin.install "deepwiki"
  end

  test do
    assert_match "deepwiki #{version}", shell_output("#{bin}/deepwiki --version")
  end
end