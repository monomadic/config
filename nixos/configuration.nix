{ config, pkgs, ... }:

{
  imports = [ /etc/nixos/hardware-configuration.nix ];

  nix = {
    package = pkgs.nixUnstable;
    extraOptions = ''
      experimental-features = nix-command flakes
    '';
  };

  nixpkgs.config.allowUnfree = true; # nvidia etc

  environment.systemPackages = with pkgs; [
    neovim
    helix # neovim alternative in rust
    lazygit # git tui
    wget
    firefox
    qutebrowser
    git
    zellij
    fd
    nnn
    lf # linux filemanager (go)
    gnupg
    pulsemixer
    acpi # battery
    ncdu # disk usage
    fzf
    ripgrep
    zip
    dotter # rust dotfiles manager
    file # filetype identification
    radare2 # reverse engineering debugger for badasses
    #cutter # gui for radare2
    killall
    htop
    btop # all the tops
    protonvpn-cli
    bat # syntax highlight for lf
    rnix-lsp # lsp language server for nix (rust)
    nixfmt
    rclone
    zig # needed for nvim lsp
    # wish # ssh keys to mnemonics
    sumneko-lua-language-server
    viu # image preview for lf
    unzip
    mpv
    cardboard
    dwl # wm
  ];

  programs.sway = {
    enable = true;
    wrapperFeatures.gtk = true; # so that gtk works properly
    extraPackages = with pkgs; [
      brightnessctl
      swaylock
      swayidle
      wl-clipboard
      wf-recorder # screen recording
      mako # notification daemon
      alacritty # term
      foot # daemon terminal
      fuzzel # launcher
      phinger-cursors
      glib # for gsettings (get rid of this?)
      vanilla-dmz
      waybar
      light # brightness
      wob # overlay bar (for volume etc)
    ];
    extraSessionCommands = ''
      export SDL_VIDEODRIVER=wayland
      export QT_QPA_PLATFORM=wayland
      export QT_WAYLAND_DISABLE_WINDOWDECORATION="1"
      export _JAVA_AWT_WM_NONREPARENTING=1
      export MOZ_ENABLE_WAYLAND=1
      export XCURSOR_SIZE=48
    '';
  };

  boot.loader = {
    systemd-boot.enable = true;
    efi.canTouchEfiVariables = true;
  };

  # services.xserver.enable = true; # enable X11
  #services.xserver.windowManager.stumpwm.enable = true;

  hardware.video.hidpi.enable = true;
  environment.variables = {
    #  GDK_SCALE = "2";
    #  GDK_DPI_SCALE = "2.0";
    #  QT_AUTO_SCREEN_SCALE_FACTOR = "1";
    XCURSOR_SIZE = "48";
    XCURSOR_THEME = "Vanilla-DMZ";
    EDITOR = "nvim";
  };
  # services.xserver.dpi = 97; # this does nothing for wayland

  programs.waybar.enable = true;

  fonts.fonts = with pkgs; [
    (nerdfonts.override { fonts = [ "DejaVuSansMono" ]; })
  ];

  # services.xserver.displayManager.gdm.enable = true;
  # services.xserver.desktopManager.gnome.enable = true;
  # services.xserver.videoDrivers = [ "nvidia" ];
  # services.xserver.layout = "us";
  # services.xserver.xkbOptions = {
  #   "eurosign:e",
  #   "caps:escape" # map caps to escape.
  # };

  # sound.enable = true;
  hardware.pulseaudio.enable = false;
  security.rtkit.enable = true;
  services.pipewire = {
    enable = true;
    alsa.enable = true;
    alsa.support32Bit = true;
    pulse.enable = true;
    #jack.enable = true;
  };

  services.xserver.libinput.enable = true; # touchpad support (default true)

  users.users.nom = {
    isNormalUser = true;
    extraGroups = [
      "wheel"
      "disk"
      "libvirtd"
      "docker"
      "audio"
      "video"
      "input"
      "systemd-journal"
      "networkmanager"
      "network"
      "davfs2"
    ]; # Enable ‘sudo’ for the user.
  };


  # services.openssh.enable = true;
  services.tlp.enable = true;
  services.yggdrasil = {
    enable = true;
    persistentKeys = false; # generate new ipv6 address each load
    config = {
      Peers = [
        # https://github.com/yggdrasil-network/public-peers
        "tcp://sin.yuetau.net:6642" # sg
        "tls://sin.yuetau.net:6643" # sg
      ];
    };
  };

  systemd = {
    targets.machines.enable = true;
    nspawn."arch" = {
      enable = true;
      execConfig = {
        Boot = true;
      };
    };
    services."systemd-nspawn@arch" = {
      enable = true;
      wantedBy = [ "machines.target" ];
    };
  };

  networking = {
    hostName = "nixos";
    # wireless.enable = true;  # wpa_supplicant.
    networkmanager.enable = true; # Easiest to use and most distros use this by default.
    # firewall.enable = false;
    # firewall.allowedTCPPorts = [ ... ];
    # firewall.allowedUDPPorts = [ ... ];
    # proxy.default = "http://user:password@proxy:port/";
    # proxy.noProxy = "127.0.0.1,localhost,internal.domain";
  };

  time.timeZone = "Asia/Bangkok";
  i18n.defaultLocale = "en_US.UTF-8";
  # console = {
  #   font = "Lat2-Terminus16";
  #   keyMap = "us";
  #   useXkbConfig = true; # use xkbOptions in tty.
  # };

  system.stateVersion = "22.05"; # determines system config state
}
