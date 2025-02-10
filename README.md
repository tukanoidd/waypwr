# What
Wayland power menu

# Why another one?
Existing implementations are either written in C/Zig/Nim or using GTK in Rust.
I'm a NixOS user, and gtk styling has always been a sore spot for me, so I wanted to have
something more reliable as a GUI backend (Iced in this case) and, of course, written in my
favorite language, which is Rust.

![Screenshot](./assets/screenshot.png)

# How work?
- GUI: [iced](https://docs.rs/iced/latest/iced/) and [iced_layershell](https://docs.rs/iced_layershell/latest/iced_layershell/)
- Actions: through DBus, leveraging [zbus](https://docs.rs/zbus/latest/zbus/) and
[logind_zbus](https://docs.rs/logind-zbus/latest/logind_zbus/)
