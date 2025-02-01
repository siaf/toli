class Toli < Formula
  # Before submitting to Homebrew, follow these steps:
  #
  # 1. Create a new GitHub release with version tag (e.g., v0.1.0)
  #
  # 2. Build release assets:
  #    cargo build --release
  #    # For macOS:
  #    tar czf toli-0.1.0-x86_64-apple-darwin.tar.gz -C target/release toli completions/
  #    # For Linux:
  #    tar czf toli-0.1.0-x86_64-unknown-linux-gnu.tar.gz -C target/release toli completions/
  #
  # 3. Upload both tar.gz files to the GitHub release
  #
  # 4. Calculate SHA256 checksums:
  #    shasum -a 256 toli-0.1.0-x86_64-apple-darwin.tar.gz
  #    shasum -a 256 toli-0.1.0-x86_64-unknown-linux-gnu.tar.gz
  #
  # 5. Replace the SHA256 placeholders below with actual values
  #
  desc "Terminal Intelligence & Learning Operator - Natural language interface for shell commands"
  homepage "https://github.com/siaf/toli"
  version "0.1.0"

  if OS.mac?
    url "https://github.com/siaf/toli/releases/download/v#{version}/toli-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "3a00d2280fdf9e5fe7af42978a737569e45edb123b4b352ace02d302a4633039"
  elsif OS.linux?
    url "https://github.com/siaf/toli/releases/download/v#{version}/toli-#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "3a00d2280fdf9e5fe7af42978a737569e45edb123b4b352ace02d302a4633039"
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