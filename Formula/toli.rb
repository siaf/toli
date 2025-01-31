class Toli < Formula
  desc "Terminal Intelligence & Learning Operator - Natural language interface for shell commands"
  homepage "https://github.com/siaf/toli"
  version "0.1.0"

  if OS.mac?
    url "https://github.com/siaf/toli/releases/download/v#{version}/toli-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "REPLACE_WITH_ACTUAL_SHA256_AFTER_RELEASE"
  elsif OS.linux?
    url "https://github.com/siaf/toli/releases/download/v#{version}/toli-#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "REPLACE_WITH_ACTUAL_SHA256_AFTER_RELEASE"
  end

  depends_on "rust" => :build

  def install
    bin.install "toli"

    # Install shell completion files
    bash_completion.install "completions/toli.bash" => "toli"
    zsh_completion.install "completions/toli.zsh" => "_toli"
    fish_completion.install "completions/toli.fish"
  end

  def caveats
    <<~EOS
      To enable command aliases, add the following to your shell configuration file:
      
      For bash (~/.bashrc):
        alias howto='toli --how'
        alias do='toli --do'
        alias explain='toli --explain'
      
      For zsh (~/.zshrc):
        alias howto='toli --how'
        alias do='toli --do'
        alias explain='toli --explain'
      
      For fish (~/.config/fish/config.fish):
        alias howto='toli --how'
        alias do='toli --do'
        alias explain='toli --explain'
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/toli --version")
  end
end