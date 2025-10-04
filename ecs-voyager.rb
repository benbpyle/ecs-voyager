class EcsVoyager < Formula
  desc "Terminal user interface (TUI) for exploring and managing AWS ECS resources"
  homepage "https://github.com/benbpyle/ecs-voyager"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/benbpyle/ecs-voyager/releases/download/v0.1.0/ecs-voyager-v0.1.0-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_ARM64_SHA256"
    else
      url "https://github.com/benbpyle/ecs-voyager/releases/download/v0.1.0/ecs-voyager-v0.1.0-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_X86_64_SHA256"
    end
  end

  def install
    bin.install "ecs-voyager"
  end

  test do
    assert_match "ecs-voyager", shell_output("#{bin}/ecs-voyager --version 2>&1", 0)
  end
end
