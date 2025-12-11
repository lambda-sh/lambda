use lambda_platform::wgpu::surface as platform_surface;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentMode {
  /// Vsync enabled; frames wait for vertical blanking interval.
  Fifo,
  /// Vsync with relaxed timing; may tear if frames miss the interval.
  FifoRelaxed,
  /// No Vsync; immediate presentation (may tear).
  Immediate,
  /// Triple-buffered presentation when supported.
  Mailbox,
  /// Automatic Vsync selection by the platform.
  AutoVsync,
  /// Automatic non-Vsync selection by the platform.
  AutoNoVsync,
}

impl PresentMode {
  pub(crate) fn to_platform(&self) -> platform_surface::PresentMode {
    match self {
      PresentMode::Fifo => platform_surface::PresentMode::Fifo,
      PresentMode::FifoRelaxed => platform_surface::PresentMode::FifoRelaxed,
      PresentMode::Immediate => platform_surface::PresentMode::Immediate,
      PresentMode::Mailbox => platform_surface::PresentMode::Mailbox,
      PresentMode::AutoVsync => platform_surface::PresentMode::AutoVsync,
      PresentMode::AutoNoVsync => platform_surface::PresentMode::AutoNoVsync,
    }
  }

  pub(crate) fn from_platform(
    mode: &platform_surface::PresentMode,
  ) -> PresentMode {
    match mode {
      platform_surface::PresentMode::Fifo => PresentMode::Fifo,
      platform_surface::PresentMode::FifoRelaxed => PresentMode::FifoRelaxed,
      platform_surface::PresentMode::Immediate => PresentMode::Immediate,
      platform_surface::PresentMode::Mailbox => PresentMode::Mailbox,
      platform_surface::PresentMode::AutoVsync => PresentMode::AutoVsync,
      platform_surface::PresentMode::AutoNoVsync => PresentMode::AutoNoVsync,
    }
  }
}

impl Default for PresentMode {
  fn default() -> Self {
    return PresentMode::Fifo;
  }
}
