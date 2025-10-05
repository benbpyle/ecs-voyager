class EcsVoyager < Formula
  desc "Terminal user interface (TUI) for exploring and managing AWS ECS resources"
  homepage "https://github.com/benbpyle/ecs-voyager"
  version "0.1.3"
  license "MIT"

  # This formula uses prebuilt binaries and does not require compilation
  def self.needs_compiler?
    false
  end

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/benbpyle/ecs-voyager/releases/download/v0.1.3/ecs-voyager-v0.1.3-aarch64-apple-darwin.tar.gz"
    sha256 "520b176da96e83a73e21ed7d3150e03aa1c8e3bbc716d410ae7d66afa139206d"
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/benbpyle/ecs-voyager/releases/download/v0.1.3/ecs-voyager-v0.1.3-x86_64-apple-darwin.tar.gz"
    sha256 "7c8b7a47b9b8bd9a3c2e01837d6f7d56da756bfc74a52be0354dd29ced213569"
  end

  def install
    bin.install "ecs-voyager"
  end

  test do
    assert_match "ecs-voyager", shell_output("#{bin}/ecs-voyager --version 2>&1", 0)
  end
end
