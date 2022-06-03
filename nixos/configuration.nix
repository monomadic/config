{ config, pkgs, ... }:

{
  nix = {
    package = pkgs.nixUnstable;
    extraOptions = ''
      		experimental-features = nix-command flakes
      	'';
  };
  imports =
    [
      # Include the results of the hardware scan.
      ./hardware-configuration.nix
    ];

  # nvidia etc
  nixpkgs.config.allowUnfree = true;

  # Use the systemd-boot EFI boot loader.
  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  # networking.hostName = "nixos"; # Define your hostname.
  # Pick only one of the below networking options.
  # networking.wireless.enable = true;  # Enables wireless support via wpa_supplicant.
  networking.networkmanager.enable = true; # Easiest to use and most distros use this by default.

  # Set your time zone.
  time.timeZone = "Asia/Bangkok";

  # Configure network proxy if necessary
  # networking.proxy.default = "http://user:password@proxy:port/";
  # networking.proxy.noProxy = "127.0.0.1,localhost,internal.domain";

  # Select internationalisation properties.
  # i18n.defaultLocale = "en_US.UTF-8";
  # console = {
  #   font = "Lat2-Terminus16";
  #   keyMap = "us";
  #   useXkbConfig = true; # use xkbOptions in tty.
  # };

  # Enable the X11 windowing system.
  # services.xserver.enable = true;
  # services.xserver.windowManager.stumpwm.enable = true;

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
      alacritty # Alacritty is the default terminal in the config
      foot # daemon terminal
      # dmenu
      fuzzel # launcher
      phinger-cursors
      # sirula # rust launcher
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
    '';
  };

  # hidpi
  hardware.video.hidpi.enable = true;
  environment.variables = {
    #  GDK_SCALE = "2";
    #  GDK_DPI_SCALE = "2.0";
    #  QT_AUTO_SCREEN_SCALE_FACTOR = "1";
    XCURSOR_SIZE = "48";
    XCURSOR_THEME = "Vanilla-DMZ";
    EDITOR = "nvim";
  };
  #services.xserver.dpi = 227;

  programs.waybar.enable = true;

  fonts.fonts = with pkgs; [
    (nerdfonts.override { fonts = [ "DejaVuSansMono" ]; })
  ];

  # Enable the GNOME Desktop Environment.
  #services.xserver.displayManager.gdm.enable = true;
  #services.xserver.desktopManager.gnome.enable = true;

  # services.xserver.videoDrivers = [ "nvidia" ];

  # Configure keymap in X11
  # services.xserver.layout = "us";
  # services.xserver.xkbOptions = {
  #   "eurosign:e";
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

  # Enable touchpad support (enabled default in most desktopManager).
  services.xserver.libinput.enable = true;

  # Define a user account. Don't forget to set a password with ‘passwd’.
  users.users.nom = {
    isNormalUser = true;
    extraGroups = [ "wheel" "disk" "libvirtd" "docker" "audio" "video" "input" "systemd-journal" "networkmanager" "network" "davfs2" ]; # Enable ‘sudo’ for the user.
  };

  environment.systemPackages = with pkgs; [
    vim
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
    # ytfzf # youtube fzf
    file # filetype identification
    #radare2 # reverse engineering debugger for badasses
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
    #wish # ssh keys to mnemonics
    sumneko-lua-language-server
    viu # image preview for lf
    radare2
    unzip
    mpv
  ];

  # Some programs need SUID wrappers, can be configured further or are
  # started in user sessions.
  # programs.mtr.enable = true;
  #
  #   enable = true;
  #   enableSSHSupport = true;
  # };

  # List services that you want to enable:
  services.tlp.enable = true;

  services.yggdrasil = {
    enable = true;
    persistentKeys = false; # generate new ipv6 address each load
    config = {
      Peers = [
        "tcp://sin.yuetau.net:6642" # sg
        "tls://sin.yuetau.net:6643" # sg
        # https://github.com/yggdrasil-network/public-peers
      ];
    };
  };

  systemd.targets.machines.enable = true;
  systemd.nspawn."arch" = {
    enable = true;
    execConfig = {
      Boot = true;
    };
  };
  systemd.services."systemd-nspawn@arch" = {
    enable = true;
    wantedBy = [ "machines.target" ];
  };


  # Enable the OpenSSH daemon.
  # services.openssh.enable = true;

  # Open ports in the firewall.
  # networking.firewall.allowedTCPPorts = [ ... ];
  # networking.firewall.allowedUDPPorts = [ ... ];
  # Or disable the firewall altogether.
  # networking.firewall.enable = false;

  # This value determines the NixOS release from which the default
  # settings for stateful data, like file locations and database versions
  # on your system were taken. It‘s perfectly fine and recommended to leave
  # this value at the release version of the first install of this system.
  # Before changing this value read the documentation for this option
  # (e.g. man configuration.nix or on https://nixos.org/nixos/options.html).
  system.stateVersion = "22.05"; # Did you read the comment?
}
