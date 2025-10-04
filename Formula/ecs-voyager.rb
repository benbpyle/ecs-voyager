class EcsVoyager < Formula
  desc "Terminal user interface (TUI) for exploring and managing AWS ECS resources"
  homepage "https://github.com/benbpyle/ecs-voyager"
  version "0.1.1"
  license "MIT"

  # This formula uses prebuilt binaries and does not require compilation
  def self.needs_compiler?
    false
  end

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/benbpyle/ecs-voyager/releases/download/v0.1.1/ecs-voyager-v0.1.1-aarch64-apple-darwin.tar.gz"
    sha256 "de34252c15fc4a0c9541090f9d90f040cd2a6de07b8175e96878d862d8f5505c"
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/benbpyle/ecs-voyager/releases/download/v0.1.1/ecs-voyager-v0.1.1-x86_64-apple-darwin.tar.gz"
    sha256 "57aba0fbd37d408af0a829fe455c1f4cdf497b5a484b3aec5ec9bb97c2944277"
  end

  def install
    bin.install "ecs-voyager"
  end

  test do
    assert_match "ecs-voyager", shell_output("#{bin}/ecs-voyager --version 2>&1", 0)
  end
end
