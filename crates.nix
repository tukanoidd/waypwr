{...}: {
  perSystem = {
    pkgs,
    config,
    ...
  }: let
    crateName = "waypwr";
  in {
    nci.projects."waypwr".path = ./.;
    nci.crates.${crateName} = {
      runtimeLibs = with pkgs;
      with xorg; [
        vulkan-loader
        wayland
        libX11
        libxkbcommon
      ];
    };
  };
}
