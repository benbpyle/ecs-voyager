class EcsVoyager < Formula
  desc "Terminal user interface (TUI) for exploring and managing AWS ECS resources"
  homepage "https://github.com/benbpyle/ecs-voyager"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/benbpyle/ecs-voyager/releases/download/v0.1.0/ecs-voyager-v0.1.0-aarch64-apple-darwin.tar.gz"
      sha256 "2bbd0358abb175848ec44da6df8437df6e966293cc0121f783f670bdb1d4c919"
    else
      url "https://github.com/benbpyle/ecs-voyager/releases/download/v0.1.0/ecs-voyager-v0.1.0-x86_64-apple-darwin.tar.gz"
      sha256 "0019dfc4b32d63c1392aa264aed2253c1e0c2fb09216f8e2cc269bbfb8bb49b5"
    end
  end

  def install
    bin.install "ecs-voyager"
  end

  test do
    assert_match "ecs-voyager", shell_output("#{bin}/ecs-voyager --version 2>&1", 0)
  end
end
